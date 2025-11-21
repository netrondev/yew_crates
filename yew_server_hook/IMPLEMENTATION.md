# Yew Server Hook Implementation

## Overview

This procedural macro bridges server-side API endpoints and client-side Yew hooks, similar to Leptos's `server_fn` macro but tailored for Yew applications.

## Key Implementation Details

### Macro Structure

The macro generates:

1. **Parameter Struct** (if function has parameters):
   - Named `{FunctionName}Params` in PascalCase
   - Derives `Serialize`, `Deserialize`, `Clone`, `Debug`

2. **Server Handler** (when `feature = "ssr"`):
   - Accepts Axum state and JSON body
   - Extracts parameters from the struct
   - Wraps return value in `axum::Json`

3. **Client Hook** (when not `feature = "ssr"`):
   - Named `use_{function_name}` (following Yew hook conventions)
   - Returns `ApiHook<T>` containing `DataState`
   - Makes POST requests using `gloo-net`
   - Handles loading, error, data, and empty states

## Usage

```rust
// Define shared types once in your app
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DataState<G> {
    Loading,
    Error(String),
    Data(G),
    Empty,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiHook<G> {
    pub state: DataState<G>,
}

// Apply the macro to your function
#[yewserverhook(path = "/api/users")]
pub async fn get_users() -> Result<Vec<User>, AppError> {
    // Implementation
}

// Use in component
#[function_component]
pub fn UserList() -> Html {
    let users = use_users(); // Generated hook

    match &users.state {
        DataState::Loading => html! { <p>{"Loading..."}</p> },
        DataState::Error(e) => html! { <p>{format!("Error: {}", e)}</p> },
        DataState::Empty => html! { <p>{"No users found"}</p> },
        DataState::Data(data) => // render data
    }
}
```

## Design Decisions

1. **Always POST**: Currently uses POST for all requests (including those without parameters) for consistency. Could be optimized to use GET for parameterless functions.

2. **Hardcoded URL**: Server URL is hardcoded to `http://localhost:3000`. Should be made configurable via environment variable or macro parameter.

3. **State Management**: Uses Yew's `use_state` and `use_effect_with` for reactive state management.

4. **Error Handling**: Comprehensive error handling at request creation, sending, and response parsing stages.

5. **Hook Naming**: Follows Yew conventions by prefixing with `use_` (e.g., `get_users` → `use_users`).

## Comparison with Leptos

| Feature | Leptos server_fn | Our yewserverhook |
|---------|------------------|-------------------|
| Conditional compilation | ✓ Uses `cfg!(feature = "ssr")` | ✓ Same approach |
| Client trait | `ServerFn` trait with `run_on_client()` | Direct hook generation |
| State management | Reactive signals | Yew hooks (`use_state`) |
| Protocol flexibility | Multiple encodings (URL, JSON, CBOR) | JSON only currently |
| Middleware support | ✓ | ✗ Not implemented |
| WebSocket support | ✓ | ✗ Not implemented |

## Future Improvements

1. **Configurable server URL** via macro parameter or environment variable
2. **HTTP method selection** (GET for queries, POST for mutations)
3. **Multiple encoding formats** (URL-encoded, MessagePack, etc.)
4. **Request caching** and deduplication
5. **Retry logic** with exponential backoff
6. **WebSocket support** for streaming responses
7. **Middleware system** for authentication, logging, etc.
8. **Better TypeScript-style type generation** for API contracts

## Testing

The macro can be tested by:
1. Building with `cargo build --examples`
2. The example in `examples/usage.rs` demonstrates both parameterless and parameterized functions
3. Real integration would require setting up an Axum server with the `ssr` feature