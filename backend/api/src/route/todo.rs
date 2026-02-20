use axum::{
    Router,
    routing::{get, patch, post},
};
use registry::AppRegistry;

use crate::handler::todo::{list_my_todos, register_todo, update_todo_completed};

pub fn build_todo_routers() -> Router<AppRegistry> {
    let routers = Router::new()
        .route("/", post(register_todo))
        .route("/me", get(list_my_todos))
        .route("/{todo_id}/completed", patch(update_todo_completed));

    Router::new().nest("/todos", routers)
}
