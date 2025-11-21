# yew_extra

[![Crates.io](https://img.shields.io/crates/v/yew_extra.svg)](https://crates.io/crates/yew_extra)
[![Documentation](https://docs.rs/yew_extra/badge.svg)](https://docs.rs/yew_extra)
[![License](https://img.shields.io/crates/l/yew_extra.svg)](https://github.com/netrondev/yew_crates)

Extract Axum request data within Yew server functions, similar to how `leptos_axum` provides extraction helpers for Leptos.

## Overview

`yew_extra` provides utilities for accessing Axum request data (headers, cookies, method, etc.) within Yew server functions when using server-side rendering. This is particularly useful when you need to access request context like cookies, headers, or custom extractors in your server functions.

## Features

- **Request Extraction**: Extract Axum request data using the `FromRequestParts` trait
- **State Support**: Compatible with extractors that require application state
- **Type-safe**: Leverages Rust's type system for compile-time guarantees
- **SSR Compatible**: Designed for server-side rendering scenarios
- **No WASM Overhead**: Server-only dependencies excluded from WASM builds

## Installation

Add this to your `Cargo.toml`:

```sh
cargo add yew_extra
```

## Usage

### Basic Extraction

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

### Extraction with State

For extractors that require application state, use `extract_with_state()`:

```rust
use yew_extra::extract_with_state;

#[yewserverhook(path = "/api/data")]
pub async fn get_data() -> Result<Data, AppError> {
    let app_state = get_app_state();
    let db_pool: DbPool = extract_with_state(&app_state).await?;

    Ok(fetch_data_from_db(db_pool).await?)
}
```

### Setting Up the Server

On the server side, you need to provide request parts before calling server functions:

```rust
use yew_extra::{provide_request_parts, clear_request_parts};
use axum::{body::Body, http::Request};

async fn handler(req: Request<Body>) {
    let (parts, body) = req.into_parts();

    // Provide the request parts to the context
    provide_request_parts(parts).await;

    // Execute your server function
    let result = your_server_function().await;

    // Clean up after completion
    clear_request_parts().await;
}
```

## How It Works

`yew_extra` uses task-local storage to make request parts available throughout the execution of a server function. When you call `provide_request_parts()`, the request data is stored with a unique task ID. The `extract()` function then retrieves this data and uses Axum's `FromRequestParts` trait to extract the desired type.

This approach is similar to how `leptos_axum` handles request extraction, making it familiar to developers coming from the Leptos ecosystem.

## Supported Extractors

Any type that implements Axum's `FromRequestParts` trait can be extracted, including:

- **HTTP Primitives**: `Method`, `Uri`, `Version`, `HeaderMap`
- **Cookies**: `CookieJar` (from `axum_extra`)
- **Headers**: `TypedHeader<T>` (from `axum_extra`)
- **Connection Info**: `ConnectInfo<T>`
- **Custom Extractors**: Any custom type implementing `FromRequestParts`

## Error Handling

Extraction can fail in two ways:

1. **MissingParts**: Request parts weren't provided (forgot to call `provide_request_parts()`)
2. **ExtractionFailed**: The extractor itself failed (e.g., missing required header)

Both errors are wrapped in the `ExtractError` enum which implements `std::error::Error`.

## Platform Support

This crate is designed for server-side use only. All server-specific dependencies are excluded from WASM builds to keep your client bundle small.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
