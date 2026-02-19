use axum::{
    Router,
    routing::{get, post},
};
use registry::AppRegistry;

use crate::handler::todo::{list_my_todos, register_todo};

pub fn build_todo_routers() -> Router<AppRegistry> {
    let routers = Router::new()
        .route("/", post(register_todo))
        .route("/me", get(list_my_todos));

    Router::new().nest("/todos", routers)
}
