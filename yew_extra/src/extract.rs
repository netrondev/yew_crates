//! Axum extractor utilities for Yew server functions.
//!
//! This module provides a way to extract Axum request parts within server functions,
//! similar to how `leptos_axum::extract()` works.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::fmt::Debug;
use std::sync::Arc;

/// Global storage for request Parts, keyed by task ID
static REQUEST_PARTS_STORAGE: Lazy<DashMap<usize, Parts>> = Lazy::new(DashMap::new);

/// Gets a unique ID for the current task
fn get_task_id() -> usize {
    // Use the thread ID as a unique identifier
    // This works because each request is typically handled on its own thread/task
    // Note: This is a simplified approach. In production, you might want a more robust solution.
    let thread_id = std::thread::current().id();
    // Hash the thread ID to get a usize
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    thread_id.hash(&mut hasher);
    hasher.finish() as usize
}

/// Error type for extraction failures
#[derive(Debug)]
pub enum ExtractError {
    /// No request parts were found in context
    MissingParts(String),
    /// Extraction failed
    ExtractionFailed(String),
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::MissingParts(msg) => write!(f, "Missing request parts: {}", msg),
            ExtractError::ExtractionFailed(msg) => write!(f, "Extraction failed: {}", msg),
        }
    }
}

impl std::error::Error for ExtractError {}

/// Provides request parts to the current context.
///
/// This should be called by the server function handler before executing the user's function.
/// The parts will be stored in task-local storage for the duration of the handler execution.
///
/// # Example
///
/// ```ignore
/// async fn handler(req: Request<Body>) {
///     let (parts, body) = req.into_parts();
///
///     provide_request_parts(parts).await;
///
///     // Now the user's function can call extract()
///     let result = user_function().await;
///
///     clear_request_parts().await;
/// }
/// ```
pub async fn provide_request_parts(parts: Parts) {
    let task_id = get_task_id();
    REQUEST_PARTS_STORAGE.insert(task_id, parts);
}

/// Clears the request parts from context.
///
/// This should be called after the server function completes to prevent memory leaks.
pub async fn clear_request_parts() {
    let task_id = get_task_id();
    REQUEST_PARTS_STORAGE.remove(&task_id);
}

/// Extracts data from the request using Axum's `FromRequestParts` trait.
///
/// This is a helper to make it easier to use Axum extractors in server functions.
/// It is generic over some type `T` that implements [`FromRequestParts`] and can
/// therefore be used in an extractor. The compiler can often infer this type.
///
/// Any error that occurs during extraction is converted to an [`ExtractError`].
///
/// # Example
///
/// ```ignore
/// use yew_extra::extract;
/// use axum::http::Method;
/// use axum_extra::extract::CookieJar;
///
/// #[yewserverhook(path = "/api/users")]
/// pub async fn get_users() -> Result<Vec<User>, AppError> {
///     // Extract the HTTP method
///     let method: Method = extract().await?;
///
///     // Extract cookies
///     let cookie_jar: CookieJar = extract().await?;
///
///     // Use the extracted data
///     let session_token = cookie_jar.get("session_token");
///
///     Ok(fetch_users().await?)
/// }
/// ```
pub async fn extract<T>() -> Result<T, ExtractError>
where
    T: Sized + FromRequestParts<()>,
    T::Rejection: Debug,
{
    extract_with_state::<T, ()>(&()).await
}

/// Extracts data from the request using Axum's `FromRequestParts` trait with state support.
///
/// This function is compatible with extractors that require access to `State`.
///
/// It is generic over some type `T` that implements [`FromRequestParts`] and can
/// therefore be used in an extractor. The compiler can often infer this type.
///
/// Any error that occurs during extraction is converted to an [`ExtractError`].
///
/// # Example
///
/// ```ignore
/// use yew_extra::extract_with_state;
///
/// #[yewserverhook(path = "/api/users")]
/// pub async fn get_users() -> Result<Vec<User>, AppError> {
///     let app_state = get_app_state();
///     let db_pool: DbPool = extract_with_state(&app_state).await?;
///
///     Ok(fetch_users_from_db(db_pool).await?)
/// }
/// ```
pub async fn extract_with_state<T, S>(state: &S) -> Result<T, ExtractError>
where
    T: Sized + FromRequestParts<S>,
    T::Rejection: Debug,
{
    let task_id = get_task_id();

    // Get the parts from storage
    let parts_ref = REQUEST_PARTS_STORAGE
        .get(&task_id)
        .ok_or_else(|| {
            ExtractError::MissingParts(
                "Request parts not found. Make sure provide_request_parts() was called.".to_string()
            )
        })?;

    // Clone the Parts (this is cheap as Parts is designed to be cloneable)
    let mut parts = parts_ref.value().clone();

    // Use from_request_parts to extract the data
    T::from_request_parts(&mut parts, state)
        .await
        .map_err(|e| ExtractError::ExtractionFailed(format!("{:?}", e)))
}
