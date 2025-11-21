use yew_server_hook::yewserverhook;

// Integration test to verify the macro expands correctly
use serde::{Deserialize, Serialize};

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

// Example with default POST method
#[yewserverhook(path = "/api/users")]
pub async fn create_user(name: String, email: String) -> Result<String, String> {
    Ok(format!("Created user: {} with email: {}", name, email))
}

// Example with GET method (no parameters)
#[yewserverhook(path = "/api/users", method = "GET")]
pub async fn list_users() -> Result<Vec<String>, String> {
    Ok(vec!["user1".to_string(), "user2".to_string()])
}

// Example with PUT method
#[yewserverhook(path = "/api/user", method = "PUT")]
pub async fn update_user(id: String, name: String) -> Result<String, String> {
    Ok(format!("Updated user {} with name: {}", id, name))
}

// Example with DELETE method
#[yewserverhook(path = "/api/user", method = "DELETE")]
pub async fn delete_user(id: String) -> Result<String, String> {
    Ok(format!("Deleted user {}", id))
}

// Example with method specified before path (either order works)
#[yewserverhook(method = "PATCH", path = "/api/user/status")]
pub async fn update_user_status(id: String, status: String) -> Result<String, String> {
    Ok(format!("Updated user {} status to: {}", id, status))
}

fn main() {
    println!("This example demonstrates the yewserverhook macro with different HTTP methods");
    println!("The macro now supports: GET, POST, PUT, DELETE, PATCH");
    println!("You can specify method and path in any order");
    println!("If method is not specified, it defaults to POST");
}
