use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Type,
};

// Define a custom parser for the macro arguments
struct MacroArgs {
    path: String,
    method: String,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path = None;
        let mut method = None;

        // Parse arguments in any order
        loop {
            if input.is_empty() {
                break;
            }

            // Parse the identifier (either "path" or "method")
            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;

            if ident == "path" {
                let path_lit: syn::LitStr = input.parse()?;
                path = Some(path_lit.value());
            } else if ident == "method" {
                let method_lit: syn::LitStr = input.parse()?;
                let method_value = method_lit.value().to_uppercase();

                // Validate the method
                if !["GET", "POST", "PUT", "DELETE", "PATCH"].contains(&method_value.as_str()) {
                    return Err(syn::Error::new(
                        method_lit.span(),
                        "Invalid HTTP method. Must be one of: GET, POST, PUT, DELETE, PATCH",
                    ));
                }
                method = Some(method_value);
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("Unknown argument '{}'. Expected 'path' or 'method'", ident),
                ));
            }

            // Check if there's a comma for more arguments
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            } else {
                break;
            }
        }

        // Path is required
        let path =
            path.ok_or_else(|| syn::Error::new(input.span(), "Missing required argument 'path'"))?;

        // Method defaults to POST if not specified
        let method = method.unwrap_or_else(|| "POST".to_string());

        Ok(MacroArgs { path, method })
    }
}

/// A procedural macro that generates both server-side API endpoint and client-side Yew hook
///

/// This will generate:
/// - A server-side handler function for use with Axum
/// - A client-side Yew hook (use_users) that fetches data from the endpoint
#[proc_macro_attribute]
pub fn yewserverhook(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // Parse the path and method arguments
    let args = parse_macro_input!(args as MacroArgs);
    let path = args.path;
    let method = args.method;

    // Extract function details
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;

    // Determine if function has parameters (excluding self)
    let has_params = !fn_inputs.is_empty();

    // Extract return type and error type
    let (return_type, error_type) = extract_return_type(fn_output);
    let error_type = error_type.unwrap_or_else(|| quote! { () });

    // Generate hook name from function name (e.g., get_users -> use_users)
    let hook_name = format!("use_{}", fn_name.to_string());
    let hook_ident = syn::Ident::new(&hook_name, fn_name.span());

    // Generate parameter struct if needed
    let param_struct = if has_params {
        generate_param_struct(fn_name, fn_inputs)
    } else {
        quote! {}
    };

    // Generate the server handler
    let server_handler = generate_server_handler(
        fn_name,
        fn_vis,
        fn_block,
        fn_inputs,
        fn_output,
        has_params,
        &return_type,
        &error_type,
        &path,
        &method,
    );

    // Generate the client hook
    let client_hook = generate_client_hook(
        &hook_ident,
        fn_vis,
        &path,
        &return_type,
        has_params,
        fn_name,
        fn_inputs,
        &method,
    );

    // Generate the direct callable function for client
    let client_function = generate_client_function(
        fn_name,
        fn_vis,
        &path,
        &return_type,
        has_params,
        fn_inputs,
        &method,
    );

    // Don't generate additional wrapper - the hook_ident is already what we want
    let hook_wrapper = quote! {};

    let expanded = quote! {

        #[cfg(feature = "ssr")]
        #input

        #param_struct

        #server_handler

        #client_hook

        #[cfg(not(feature = "ssr"))]
        #client_function

        #hook_wrapper
    };

    TokenStream::from(expanded)
}

fn extract_return_type(
    output: &ReturnType,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    match output {
        ReturnType::Default => (quote! { () }, None),
        ReturnType::Type(_, ty) => {
            // Extract the inner type from Result<T, E>
            if let Type::Path(type_path) = &**ty {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Result" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let (
                                Some(syn::GenericArgument::Type(ok_type)),
                                Some(syn::GenericArgument::Type(err_type)),
                            ) = (args.args.first(), args.args.iter().nth(1))
                            {
                                return (quote! { #ok_type }, Some(quote! { #err_type }));
                            }
                        }
                    }
                }
            }
            (quote! { #ty }, None)
        }
    }
}

fn generate_param_struct(
    fn_name: &syn::Ident,
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let struct_name = syn::Ident::new(
        &format!("{}Params", to_pascal_case(&fn_name.to_string())),
        fn_name.span(),
    );

    let mut fields = Vec::new();

    for input in inputs {
        if let FnArg::Typed(pat_type) = input {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let field_name = &pat_ident.ident;
                let field_type = &pat_type.ty;
                fields.push(quote! {
                    pub #field_name: #field_type
                });
            }
        }
    }

    quote! {
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
        pub struct #struct_name {
            #(#fields),*
        }
    }
}

fn generate_server_handler(
    fn_name: &syn::Ident,
    vis: &syn::Visibility,
    block: &syn::Block,
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    _output: &ReturnType,
    has_params: bool,
    return_type: &proc_macro2::TokenStream,
    error_type: &proc_macro2::TokenStream,
    path: &str,
    method: &str,
) -> proc_macro2::TokenStream {
    let fn_handler_name =
        syn::Ident::new(&format!("{}_handler", fn_name.to_string()), fn_name.span());

    let params_arg = if has_params {
        let struct_name = syn::Ident::new(
            &format!("{}Params", to_pascal_case(&fn_name.to_string())),
            fn_name.span(),
        );
        // Use Query for GET, Json for other methods
        if method == "GET" {
            quote! { axum::extract::Query(params): axum::extract::Query<#struct_name>, }
        } else {
            quote! { axum::Json(params): axum::Json<#struct_name>, }
        }
    } else {
        quote! {}
    };

    let param_extraction = if has_params {
        let mut field_names = Vec::new();
        for input in inputs {
            if let FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    field_names.push(&pat_ident.ident);
                }
            }
        }
        let struct_name = syn::Ident::new(
            &format!("{}Params", to_pascal_case(&fn_name.to_string())),
            fn_name.span(),
        );
        quote! {
            let #struct_name { #(#field_names),* } = params;
        }
    } else {
        quote! {}
    };

    // Create a modified function body that extracts parameters and wraps return in Json
    let original_stmts = &block.stmts;
    let modified_block = quote! {
        {
            #param_extraction

            // Original function body
            let result: Result<#return_type, #error_type> = async {
                #(#original_stmts)*
            }.await;

            // Wrap successful result in Json
            result.map(axum::Json)
        }
    };

    // Generate a wrapper function that converts Request<Body> to the handler's expected format
    let wrapper_fn_name = syn::Ident::new(
        &format!("{}_wrapper", fn_handler_name),
        fn_handler_name.span(),
    );

    // Generate the extraction logic based on method and whether there are params
    let extract_and_call = if has_params {
        let struct_name = syn::Ident::new(
            &format!("{}Params", to_pascal_case(&fn_name.to_string())),
            fn_name.span(),
        );

        if method == "GET" {
            // Extract query parameters for GET
            quote! {
                use ::axum::extract::FromRequestParts;

                let (mut parts, _body) = req.into_parts();

                // Provide parts to yew_extra context before calling the handler
                ::yew_extra::provide_request_parts(parts.clone()).await;

                let result = match ::axum::extract::Query::<#struct_name>::from_request_parts(&mut parts, &()).await {
                    Ok(::axum::extract::Query(params)) => {
                        let response = #fn_handler_name(::axum::extract::Query(params)).await;
                        response.into_response()
                    },
                    Err(e) => {
                        ::axum::http::Response::builder()
                            .status(::axum::http::StatusCode::BAD_REQUEST)
                            .body(::axum::body::Body::from(format!("Invalid query parameters: {}", e)))
                            .unwrap()
                    }
                };

                // Clear parts after handler completes
                ::yew_extra::clear_request_parts().await;
                result
            }
        } else {
            // Extract JSON body for POST/PUT/DELETE/PATCH
            quote! {
                use ::axum::extract::FromRequest;

                let (parts, body) = req.into_parts();

                // Provide parts to yew_extra context before calling the handler
                ::yew_extra::provide_request_parts(parts.clone()).await;

                let req = ::axum::http::Request::from_parts(parts, body);

                let result = match ::axum::Json::<#struct_name>::from_request(req, &()).await {
                    Ok(params) => {
                        let response = #fn_handler_name(params).await;
                        response.into_response()
                    },
                    Err(e) => {
                        ::axum::http::Response::builder()
                            .status(::axum::http::StatusCode::BAD_REQUEST)
                            .body(::axum::body::Body::from(format!("Invalid request: {}", e)))
                            .unwrap()
                    }
                };

                // Clear parts after handler completes
                ::yew_extra::clear_request_parts().await;
                result
            }
        }
    } else {
        quote! {
            // No parameters, but still provide Parts for extraction
            let (parts, _body) = req.into_parts();

            // Provide parts to yew_extra context before calling the handler
            ::yew_extra::provide_request_parts(parts).await;

            let response = #fn_handler_name().await;

            // Clear parts after handler completes
            ::yew_extra::clear_request_parts().await;

            response.into_response()
        }
    };

    // Convert method string to TokenStream identifier
    let method_ident = syn::Ident::new(&method, proc_macro2::Span::call_site());

    // Generate inventory submission for automatic registration
    // This creates a wrapper that can work with raw Request<Body>
    // The inventory submission is only for non-test builds
    let inventory_submission = quote! {
        // Only generate the wrapper and inventory submission in non-test builds
        #[cfg(all(feature = "ssr", not(test)))]
        fn #wrapper_fn_name(
            req: ::axum::http::Request<::axum::body::Body>
        ) -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ::axum::http::Response<::axum::body::Body>> + Send>> {
            Box::pin(async move {
                use ::axum::response::IntoResponse;
                #extract_and_call
            })
        }

        #[cfg(all(feature = "ssr", not(test)))]
        ::inventory::submit! {
            crate::route_registry::RouteInfo::new(
                #path,
                ::axum::http::Method::#method_ident,
                #wrapper_fn_name
            )
        }
    };

    quote! {
        #[cfg(feature = "ssr")]
        #vis async fn #fn_handler_name(
            #params_arg
            // axum::extract::State(state): axum::extract::State<AppState>
        ) -> Result<axum::Json<#return_type>, #error_type> #modified_block

        #inventory_submission
    }
}

fn generate_client_function(
    fn_name: &syn::Ident,
    vis: &syn::Visibility,
    path: &str,
    return_type: &proc_macro2::TokenStream,
    has_params: bool,
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    method: &str,
) -> proc_macro2::TokenStream {
    // let host_url = quote! { "http://localhost:4000" };
    let host_url = quote! { "" };

    // Generate function parameters
    let func_params = if has_params {
        let mut params = Vec::new();
        for input in inputs {
            if let FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let param_name = &pat_ident.ident;
                    let param_type = &pat_type.ty;
                    params.push(quote! { #param_name: #param_type });
                }
            }
        }
        quote! { #(#params),* }
    } else {
        quote! {}
    };

    // Convert method to lowercase for gloo_net
    let method_lower = method.to_lowercase();
    let method_fn = syn::Ident::new(&method_lower, proc_macro2::Span::call_site());

    // Generate request body creation
    let request_body = if has_params && method != "GET" {
        let struct_name = syn::Ident::new(
            &format!("{}Params", to_pascal_case(&fn_name.to_string())),
            fn_name.span(),
        );
        let mut field_names = Vec::new();
        for input in inputs {
            if let FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    field_names.push(&pat_ident.ident);
                }
            }
        }
        quote! {
            let params = #struct_name {
                #(#field_names),*
            };
            let body = serde_json::to_string(&params)
                .map_err(|e| format!("Failed to serialize parameters: {}", e))?;

            let request = gloo_net::http::Request::#method_fn(&format!("{}{}", #host_url, #path))
                .header("Content-Type", "application/json")
                .body(body)
                .map_err(|e| format!("Failed to create request: {}", e))?;
        }
    } else if has_params && method == "GET" {
        // Build query string for GET requests
        let struct_name = syn::Ident::new(
            &format!("{}Params", to_pascal_case(&fn_name.to_string())),
            fn_name.span(),
        );
        let mut field_names = Vec::new();
        for input in inputs {
            if let FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    field_names.push(&pat_ident.ident);
                }
            }
        }
        quote! {
            let params = #struct_name {
                #(#field_names),*
            };

            // Serialize to query string
            let query_string = serde_urlencoded::to_string(&params)
                .map_err(|e| format!("Failed to serialize query parameters: {}", e))?;

            let url = format!("{}{}?{}", #host_url, #path, query_string);

            let request = gloo_net::http::Request::#method_fn(&url)
                .header("Content-Type", "application/json");
        }
    } else {
        quote! {
            let request = gloo_net::http::Request::#method_fn(&format!("{}{}", #host_url, #path))
                .header("Content-Type", "application/json");
        }
    };

    // Generate the function name for the direct call version
    let async_fn_name = syn::Ident::new(&format!("{}", fn_name.to_string()), fn_name.span());

    quote! {
        #[cfg(not(feature = "ssr"))]
        #vis async fn #async_fn_name(#func_params) -> Result<#return_type, String> {
            #request_body

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to fetch data: {}", e))?;

            // Check if the response status is successful (2xx)
            if response.ok() {
                response
                    .json::<#return_type>()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))
            } else {
                // Handle error response - try to get the error message from the response
                let status = response.status();
                let error_msg = match response.text().await {
                    Ok(text) => {
                        // Try to parse as JSON error message
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(msg) = json.get("error").and_then(|v| v.as_str()) {
                                msg.to_string()
                            } else if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
                                msg.to_string()
                            } else {
                                text
                            }
                        } else {
                            text
                        }
                    }
                    Err(_) => format!("Request failed with status {}", status)
                };
                Err(error_msg)
            }
        }
    }
}

fn generate_client_hook(
    hook_name: &syn::Ident,
    vis: &syn::Visibility,
    path: &str,
    return_type: &proc_macro2::TokenStream,
    has_params: bool,
    fn_name: &syn::Ident,
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    method: &str,
) -> proc_macro2::TokenStream {
    // let host_url = quote! { "http://localhost:4000" };
    let host_url = quote! { "" };

    let hook_params = if has_params {
        let mut params = Vec::new();
        for input in inputs {
            if let FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let param_name = &pat_ident.ident;
                    let param_type = &pat_type.ty;
                    params.push(quote! { #param_name: #param_type });
                }
            }
        }
        quote! { #(#params),* }
    } else {
        quote! {}
    };

    // Convert method to lowercase for gloo_net
    let method_lower = method.to_lowercase();
    let method_fn = syn::Ident::new(&method_lower, proc_macro2::Span::call_site());

    let request_body = if has_params && method != "GET" {
        let struct_name = syn::Ident::new(
            &format!("{}Params", to_pascal_case(&fn_name.to_string())),
            fn_name.span(),
        );
        let mut field_names = Vec::new();
        for input in inputs {
            if let FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    field_names.push(&pat_ident.ident);
                }
            }
        }
        quote! {
            let params = #struct_name {
                #(#field_names: #field_names.clone()),*
            };
            let body = serde_json::to_string(&params).unwrap();
            let request = match gloo_net::http::Request::#method_fn(
                &format!("{}{}", #host_url, #path)
            )
            .header("Content-Type", "application/json")
            .body(body) {
                Ok(req) => req,
                Err(e) => {
                    state.set(DataState::Error(format!("Failed to create request: {}", e)));
                    return;
                }
            };
        }
    } else if has_params && method == "GET" {
        // Build query string for GET requests
        let struct_name = syn::Ident::new(
            &format!("{}Params", to_pascal_case(&fn_name.to_string())),
            fn_name.span(),
        );
        let mut field_names = Vec::new();
        for input in inputs {
            if let FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    field_names.push(&pat_ident.ident);
                }
            }
        }
        quote! {
            let params = #struct_name {
                #(#field_names: #field_names.clone()),*
            };
            let query_string = match serde_urlencoded::to_string(&params) {
                Ok(qs) => qs,
                Err(e) => {
                    state.set(DataState::Error(format!("Failed to serialize query parameters: {}", e)));
                    return;
                }
            };
            let request = gloo_net::http::Request::#method_fn(
                &format!("{}{}?{}", #host_url, #path, query_string)
            )
            .header("Content-Type", "application/json");
        }
    } else {
        quote! {
            let request = gloo_net::http::Request::#method_fn(
                &format!("{}{}", #host_url, #path)
            )
            .header("Content-Type", "application/json");
        }
    };

    let deps = if has_params {
        let mut dep_names = Vec::new();
        for input in inputs {
            if let FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    dep_names.push(&pat_ident.ident);
                }
            }
        }
        quote! { (#(#dep_names.clone()),*) }
    } else {
        quote! { () }
    };

    // Check if return type looks like a Vec
    let is_vec = quote!(#return_type).to_string().contains("Vec");

    let data_handling = if is_vec {
        quote! {
            if fetched_data.is_empty() {
                state.set(DataState::Empty);
            } else {
                state.set(DataState::Data(fetched_data));
            }
        }
    } else {
        quote! {
            state.set(DataState::Data(fetched_data));
        }
    };

    quote! {

        #[cfg(feature = "ssr")]
        #[yew::hook]
        #vis fn #hook_name(#hook_params) -> ApiHook<#return_type> {
            let state = yew::use_state(|| DataState::<#return_type>::Loading);

            let is_loading = yew::use_state(|| false);
            let is_updating = yew::use_state(|| false);

            ApiHook {
                state: (*state).clone(),
                is_loading: (*is_loading).clone(),
                is_updating: (*is_updating).clone(),
            }
        }

        #[cfg(not(feature = "ssr"))]
        #[yew::hook]
        #vis fn #hook_name(#hook_params) -> ApiHook<#return_type> {
            let state = yew::use_state(|| DataState::<#return_type>::Loading);

            let is_loading = yew::use_state(|| false);
            let is_updating = yew::use_state(|| false);

            {
                let state = state.clone();
                let is_loading = is_loading.clone();
                let is_updating = is_updating.clone();

                yew::use_effect_with(#deps, move |_| {
                    // Check if this is the first load
                    let is_first_load = matches!(*state, DataState::Loading);

                    // Set appropriate loading flag
                    if is_first_load {
                        is_loading.set(true);
                        is_updating.set(true);
                    } else {
                        is_updating.set(true);
                    }

                    wasm_bindgen_futures::spawn_local(async move {
                        #request_body

                        match request.send().await {
                            Ok(response) => {
                                // Check if the response status is successful (2xx)
                                if response.ok() {
                                    match response.json::<#return_type>().await {
                                        Ok(fetched_data) => {
                                            #data_handling
                                        }
                                        Err(e) => {
                                            state.set(DataState::Error(format!(
                                                "Failed to parse response: {}",
                                                e
                                            )));
                                        }
                                    }
                                } else {
                                    // Handle error response - try to get the error message from the response
                                    let status = response.status();
                                    let error_msg = match response.text().await {
                                        Ok(text) => {
                                            // Try to parse as JSON error message
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                                if let Some(msg) = json.get("error").and_then(|v| v.as_str()) {
                                                    msg.to_string()
                                                } else if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
                                                    msg.to_string()
                                                } else {
                                                    text
                                                }
                                            } else {
                                                text
                                            }
                                        }
                                        Err(_) => format!("Request failed with status {}", status)
                                    };
                                    state.set(DataState::Error(error_msg));
                                }
                            }
                            Err(e) => {
                                state.set(DataState::Error(format!(
                                    "Failed to fetch data: {}",
                                    e
                                )));
                            }
                        }

                        // Clear loading flags after request completes
                        is_loading.set(false);
                        is_updating.set(false);
                    });
                    || ()
                });
            }

            ApiHook {
                state: (*state).clone(),
                is_loading: *is_loading,
                is_updating: *is_updating,
            }
        }
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
