// This is an example showing how the yewserverhook macro works
// It demonstrates the usage pattern you requested

use serde::{Deserialize, Serialize};
use yew_server_hook::yewserverhook;

// Define the shared types that the macro expects
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

// Mock server-side types for the example
#[cfg(feature = "ssr")]
pub mod api {
    pub struct AppState;
}

#[cfg(feature = "ssr")]
pub mod apperror {
    pub use super::AppError;
}

// RecordId type for the example
pub type RecordId = String;

// AppError type for the example
#[derive(Debug)]
#[allow(dead_code)]
pub struct AppError(String);

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError(s.to_string())
    }
}

// User struct as in your example
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: RecordId,
    pub name: Option<String>,
    pub email: Option<String>,
}

// This macro expands to both server and client code
#[yewserverhook(path = "/api/users")]
pub async fn get_users() -> Result<Vec<User>, AppError> {
    // Hardcoded data instead of database
    let users = vec![
        User {
            id: "user_001".to_string(),
            name: Some("Alice Johnson".to_string()),
            email: Some("alice@example.com".to_string()),
        },
        User {
            id: "user_002".to_string(),
            name: Some("Bob Smith".to_string()),
            email: Some("bob@example.com".to_string()),
        },
        User {
            id: "user_003".to_string(),
            name: Some("Charlie Brown".to_string()),
            email: None,
        },
    ];

    Ok(users)
}

// Usage in a Yew component
#[yew::function_component]
pub fn GetUsers() -> yew::Html {
    // The macro generates this function that returns ApiHook<Vec<User>>
    let usersapi = use_get_users();

    yew::html! {
        <div class="container mx-auto p-6">
            <h1 class="text-3xl font-bold mb-6">{ "Users List" }</h1>
            { match &usersapi.state {
                DataState::Loading => yew::html! {
                    <div class="text-center p-8">
                        <p class="text-lg text-gray-600">{ "Loading users..." }</p>
                    </div>
                },
                DataState::Error(err) => yew::html! {
                    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                        <p>
                            <strong>{ "Error: " }</strong>
                            { err }
                        </p>
                    </div>
                },
                DataState::Empty => yew::html! {
                    <div class="bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded">
                        <p>{ "No users found in the database." }</p>
                    </div>
                },
                DataState::Data(data) => yew::html! {
                    <div class="bg-white shadow-md rounded-lg overflow-hidden">
                        <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        { "ID" }
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        { "Name" }
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        { "Email" }
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                { data.iter().map(|user| {
                                    yew::html! {
                                        <tr class="hover:bg-gray-50">
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                {&user.id.to_string()}
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                {user.name.as_ref().unwrap_or(&"N/A".to_string())}
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                {user.email.as_ref().unwrap_or(&"N/A".to_string())}
                                            </td>
                                        </tr>
                                    }
                                }).collect::<yew::Html>() }
                            </tbody>
                        </table>
                    </div>
                }
            }}
        </div>
    }
}

// Example with parameters
#[yewserverhook(path = "/api/user")]
pub async fn get_user_by_id(id: String) -> Result<User, AppError> {
    // Hardcoded data lookup instead of database
    let users = vec![
        User {
            id: "user_001".to_string(),
            name: Some("Alice Johnson".to_string()),
            email: Some("alice@example.com".to_string()),
        },
        User {
            id: "user_002".to_string(),
            name: Some("Bob Smith".to_string()),
            email: Some("bob@example.com".to_string()),
        },
        User {
            id: "user_003".to_string(),
            name: Some("Charlie Brown".to_string()),
            email: None,
        },
    ];

    users
        .into_iter()
        .find(|u| u.id == id)
        .ok_or_else(|| AppError(format!("User with id {} not found", id)))
}

// Component that uses the parameterized hook
#[derive(yew::Properties, PartialEq)]
pub struct UserDetailsProps {
    pub user_id: String,
}

#[yew::function_component]
pub fn UserDetails(props: &UserDetailsProps) -> yew::Html {
    let user_id = props.user_id.clone();
    // The macro generates use_user_by_id hook
    let user_api = use_get_user_by_id(user_id.clone());

    // Create onclick handler for the button
    let onclick = {
        let user_id = user_id.clone();
        yew::Callback::from(move |_| {
            let user_id = user_id.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Call the async function generated by the macro
                match get_user_by_id(user_id).await {
                    Ok(_user) => {
                        // // Log the result to console
                        // web_sys::console::log_1(&format!("User fetched successfully: {:?}", user).into());
                        // web_sys::console::log_1(&format!("User ID: {}", user.id).into());
                        // if let Some(name) = &user.name {
                        //     web_sys::console::log_1(&format!("User Name: {}", name).into());
                        // }
                        // if let Some(email) = &user.email {
                        //     web_sys::console::log_1(&format!("User Email: {}", email).into());
                        // }
                    }
                    Err(_err) => {
                        // web_sys::console::error_1(&format!("Error fetching user: {}", err).into());
                    }
                }
            });
        })
    };

    yew::html! {
        <div class="container mx-auto p-6">

            <button {onclick} class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
                {"Get User by ID"}
            </button>

            { match &user_api.state {
                DataState::Loading => yew::html! {
                    <div class="text-center p-8">
                        <p class="text-lg text-gray-600">{ "Loading user..." }</p>
                    </div>
                },
                DataState::Error(err) => yew::html! {
                    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                        <p>
                            <strong>{ "Error: " }</strong>
                            { err }
                        </p>
                    </div>
                },
                DataState::Empty => yew::html! {
                    <div class="bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded">
                        <p>{ "User not found." }</p>
                    </div>
                },
                DataState::Data(user) => yew::html! {
                    <div class="bg-white shadow-md rounded-lg p-6">
                        <h2 class="text-2xl font-bold mb-4">{ "User Details" }</h2>
                        <div class="space-y-2">
                            <p>
                                <strong>{ "ID: " }</strong>
                                { &user.id }
                            </p>
                            <p>
                                <strong>{ "Name: " }</strong>
                                { user.name.as_ref().unwrap_or(&"N/A".to_string()) }
                            </p>
                            <p>
                                <strong>{ "Email: " }</strong>
                                { user.email.as_ref().unwrap_or(&"N/A".to_string()) }
                            </p>
                        </div>
                    </div>
                }
            }}
        </div>
    }
}

fn main() {
    println!("This is just an example demonstrating the macro usage");
    println!("The macro generates:");
    println!("1. Server-side handler function for Axum");
    println!("2. Client-side Yew hook (use_users)");
    println!("3. Wrapper function (get_users) that calls the hook");
    println!("4. DataState and ApiHook types for state management");
}
