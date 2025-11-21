# yew_crates

Opensource crates for working with rust and [yew](https://yew.rs/)


# yew_server_hook

A procedural macro that generates both server-side API handlers and client-side Yew hooks from a single function definition. Write your backend logic once, and automatically get both the Axum server endpoint and the Yew hook to call it.

[![Crates.io](https://img.shields.io/crates/v/yew_server_hook.svg)](https://crates.io/crates/yew_server_hook)
[![Documentation](https://docs.rs/yew_server_hook/badge.svg)](https://docs.rs/yew_server_hook)
[![License](https://img.shields.io/crates/l/yew_server_hook.svg)](https://github.com/netrondev/yew_crates)


```rust
use yew_server_hook::yewserverhook;

#[yewserverhook(path = "/api/hello", method = "GET")]
pub async fn get_hello() -> Result<String, String> {
    Ok("Hello, World!".to_string())
}
```

# yew_extra

Extract Axum request data within Yew server functions, similar to how `leptos_axum` provides extraction helpers for Leptos.

[![Crates.io](https://img.shields.io/crates/v/yew_extra.svg)](https://crates.io/crates/yew_extra)
[![Documentation](https://docs.rs/yew_extra/badge.svg)](https://docs.rs/yew_extra)
[![License](https://img.shields.io/crates/l/yew_extra.svg)](https://github.com/netrondev/yew_crates)

Use the `extract()` function to access request data in your server functions:

```rust
use yew_extra::extract;
use axum::http::Method;
use axum_extra::extract::CookieJar;

#[yewserverhook(path = "/api/users")]
pub async fn get_users() -> Result<Vec<User>, AppError> {
    // Extract the HTTP method
    let method: Method = extract().await?;

    // Extract cookies
    let cookie_jar: CookieJar = extract().await?;

    // Use the extracted data
    if let Some(session) = cookie_jar.get("session_token") {
        // Validate session...
    }

    Ok(fetch_users().await?)
}
```