# yew_server_hook

[![Crates.io](https://img.shields.io/crates/v/yew_server_hook.svg)](https://crates.io/crates/yew_server_hook)
[![Documentation](https://docs.rs/yew_server_hook/badge.svg)](https://docs.rs/yew_server_hook)
[![License](https://img.shields.io/crates/l/yew_server_hook.svg)](https://github.com/netrondev/yew_crates)

A procedural macro that generates both server-side API handlers and client-side Yew hooks from a single function definition. Write your backend logic once, and automatically get both the Axum server endpoint and the Yew hook to call it.

## Features

- **Single Source of Truth**: Define your API logic once, use it everywhere
- **Type Safety**: Full type checking between client and server
- **Automatic Serialization**: Handles JSON serialization/deserialization automatically
- **Multiple HTTP Methods**: Support for GET, POST, PUT, DELETE, and PATCH
- **Smart Parameter Handling**: Query parameters for GET, JSON body for other methods
- **Loading States**: Built-in loading and updating state management
- **Error Handling**: Automatic error propagation and parsing
- **SSR Support**: Server-side rendering compatible with feature flags
- **Auto Route Registration**: Routes are automatically registered using the inventory crate

## Installation

Add this to your `Cargo.toml`:

```sh
cargo add yew_server_hook
```

## Quick Start

### Basic Example

```rust
use yew_server_hook::yewserverhook;

#[yewserverhook(path = "/api/hello", method = "GET")]
pub async fn get_hello() -> Result<String, String> {
    Ok("Hello, World!".to_string())
}
```

This generates:
- A server handler at `/api/hello` (GET)
- A Yew hook `use_get_hello()`
- A direct callable function `get_hello()` for programmatic use

### Using the Hook in a Component

```rust
use yew::prelude::*;

#[function_component(HelloComponent)]
pub fn hello_component() -> Html {
    let data = use_get_hello();

    match &data.state {
        DataState::Loading => html! { <p>{"Loading..."}</p> },
        DataState::Data(message) => html! { <p>{message}</p> },
        DataState::Error(err) => html! { <p class="error">{err}</p> },
        DataState::Empty => html! { <p>{"No data"}</p> },
    }
}
```

### Example with Parameters

```rust
#[yewserverhook(path = "/api/users", method = "GET")]
pub async fn get_users(role: String, active: bool) -> Result<Vec<User>, String> {
    // Your database logic here
    let users = fetch_users_from_db(role, active).await?;
    Ok(users)
}
```

Use in a component:

```rust
#[function_component(UsersComponent)]
pub fn users_component() -> Html {
    let users = use_get_users("admin".to_string(), true);

    html! {
        <div>
            if users.is_loading {
                <p>{"Loading users..."}</p>
            }
            {
                match &users.state {
                    DataState::Data(user_list) => html! {
                        <ul>
                            { for user_list.iter().map(|user| html! {
                                <li>{&user.name}</li>
                            })}
                        </ul>
                    },
                    DataState::Error(err) => html! { <p>{err}</p> },
                    _ => html! {}
                }
            }
        </div>
    }
}
```

### POST Request Example

```rust
#[yewserverhook(path = "/api/users", method = "POST")]
pub async fn create_user(name: String, email: String) -> Result<User, String> {
    // Your database logic here
    let user = User { name, email };
    save_user_to_db(&user).await?;
    Ok(user)
}

// Direct call (not using the hook):
async fn handle_submit() {
    match create_user("Alice".to_string(), "alice@example.com".to_string()).await {
        Ok(user) => log::info!("Created user: {:?}", user),
        Err(e) => log::error!("Error: {}", e),
    }
}
```

## HTTP Methods

The macro supports all standard HTTP methods:

- `GET` - Parameters sent as query strings
- `POST` - Parameters sent as JSON body (default)
- `PUT` - Parameters sent as JSON body
- `DELETE` - Parameters sent as JSON body
- `PATCH` - Parameters sent as JSON body

## API Hook State

The generated hook returns an `ApiHook<T>` struct with:

```rust
pub struct ApiHook<T> {
    pub state: DataState<T>,
    pub is_loading: bool,    // True only on first load
    pub is_updating: bool,   // True on first load and subsequent updates
}
```

The `DataState<T>` enum:

```rust
pub enum DataState<T> {
    Loading,           // Initial state
    Data(T),          // Successfully loaded data
    Error(String),    // Error with message
    Empty,            // For Vec types, when response is empty
}
```

## Features

### SSR Feature Flag

The crate supports server-side rendering through the `ssr` feature flag:

```toml
[features]
ssr = []
```

- When `ssr` is enabled: Server handlers are generated
- When `ssr` is disabled: Client-side hooks and fetch functions are generated

## Route Registration

Routes are automatically registered using the `inventory` crate. To use the auto-registered routes:

```rust
use axum::Router;

// Routes are automatically collected and can be registered
let app = Router::new()
    .merge(your_generated_routes());
```

## Requirements

Your function must:
- Be `async`
- Return a `Result<T, E>` where both `T` and `E` implement `Serialize` and `Deserialize`
- Have parameters that implement `Serialize`, `Deserialize`, and `Clone`

## Generated Code

For each annotated function, the macro generates:

1. **Parameter Struct** (if function has parameters):
   ```rust
   #[derive(Debug, Serialize, Deserialize, Clone)]
   pub struct FunctionNameParams {
       pub param1: Type1,
       pub param2: Type2,
   }
   ```

2. **Server Handler** (with `ssr` feature):
   ```rust
   #[cfg(feature = "ssr")]
   pub async fn function_name_handler(
       axum::Json(params): axum::Json<FunctionNameParams>
   ) -> Result<axum::Json<ReturnType>, ErrorType>
   ```

3. **Client Hook**:
   ```rust
   #[yew::hook]
   pub fn use_function_name(params...) -> ApiHook<ReturnType>
   ```

4. **Direct Client Function**:
   ```rust
   pub async fn function_name(params...) -> Result<ReturnType, String>
   ```

## Testing

Run tests with:

```bash
cargo test --tests
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
