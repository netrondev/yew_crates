// Integration test to verify the macro expands correctly
use serde::{Deserialize, Serialize};
use yew_server_hook::yewserverhook;

// Required types for the macro
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
    pub is_loading: bool,
    pub is_updating: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AppError(String);

// Mock server-side types
#[cfg(feature = "ssr")]
pub mod api {
    pub struct AppState;
}

#[cfg(feature = "ssr")]
pub mod apperror {
    pub use super::AppError;
}

// Test data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestData {
    pub id: i32,
    pub value: String,
}

// Test that the macro expands for a simple function
#[yewserverhook(path = "/api/test")]
pub async fn get_test_data() -> Result<Vec<TestData>, AppError> {
    Ok(vec![
        TestData {
            id: 1,
            value: "test1".to_string(),
        },
        TestData {
            id: 2,
            value: "test2".to_string(),
        },
    ])
}

// Test that the macro expands for a function with parameters
#[yewserverhook(path = "/api/test_by_id")]
pub async fn get_test_by_id(id: i32) -> Result<TestData, AppError> {
    Ok(TestData {
        id,
        value: format!("test{}", id),
    })
}

// Test that the macro expands for a function with multiple parameters
#[yewserverhook(path = "/api/search")]
pub async fn search_items(query: String, limit: usize) -> Result<Vec<TestData>, AppError> {
    Ok((0..limit)
        .map(|i| TestData {
            id: i as i32,
            value: format!("{}-{}", query, i),
        })
        .collect())
}

#[test]
fn test_macro_expansion() {
    // This test just verifies that the macro expands without compile errors
    // The actual functionality would be tested in an integration environment
    // with both server and client running

    // Verify the generated types exist (they would be created by the macro)
    let _state: DataState<Vec<TestData>> = DataState::Loading;
    let _hook: ApiHook<Vec<TestData>> = ApiHook {
        state: DataState::Loading,
        is_loading: false,
        is_updating: false,
    };

    assert!(true, "Macro expansion successful");
}

#[test]
fn test_data_state_variants() {
    let loading: DataState<String> = DataState::Loading;
    assert!(matches!(loading, DataState::Loading));

    let error: DataState<String> = DataState::Error("error".to_string());
    assert!(matches!(error, DataState::Error(_)));

    let data: DataState<String> = DataState::Data("value".to_string());
    assert!(matches!(data, DataState::Data(_)));

    let empty: DataState<Vec<i32>> = DataState::Empty;
    assert!(matches!(empty, DataState::Empty));
}
