use axum::{Router, routing::post};
use registry::AppRegistry;

use crate::handler::todo::register_todo;

pub fn build_todo_routers() -> Router<AppRegistry> {
    let routers = Router::new().route("/", post(register_todo));

    Router::new().nest("/todos", routers)
}
