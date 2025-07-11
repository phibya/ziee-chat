#[tauri::command]
pub fn get_http_port() -> u16 {
    *crate::HTTP_PORT
}

pub fn is_desktop_app() -> bool {
    std::env::var("HEADLESS").unwrap_or_default() != "true"
}

// #[debug_handler]
// pub fn is_initializing() -> (StatusCode, String) {
//     //check if the root user is not created yet, if yes, then the app is initializing
//     (
//         StatusCode::OK,
//         match crate::database::get_database_pool() {
//             Some(pool) => {
//                 let user = UserQueries::get_root_user(&pool);
//                 serde_json::to_string(&!user.is_some()).unwrap_or_else(|_| "\"false\"".to_string())
//             }
//             None => {
//                 //throw an error if the database pool is not initialized
//                 eprintln!("Database pool is not initialized");
//                 serde_json::to_string(&false).unwrap_or_else(|_| "\"false\"".to_string())
//             }
//         },
//     )
// }
