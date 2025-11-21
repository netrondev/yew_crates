//! # yew_extra
//!
//! Utilities for Yew server-side rendering with Axum integration.
//!
//! This crate provides helpers to extract Axum request data within Yew server functions,
//! similar to how `leptos_axum` provides extraction helpers for Leptos.

#![cfg_attr(not(target_arch = "wasm32"), allow(unused_imports))]

#[cfg(not(target_arch = "wasm32"))]
mod extract;

#[cfg(not(target_arch = "wasm32"))]
pub use extract::{extract, extract_with_state, provide_request_parts, clear_request_parts};

// Re-export commonly used types for convenience
#[cfg(not(target_arch = "wasm32"))]
pub use axum::http::request::Parts;
