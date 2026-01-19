use axum::{Router, routing::post};
use registry::AppRegistry;

use crate::handler::user::{list_users, register_user};

pub fn build_user_routers() -> Router<AppRegistry> {
    let routers = Router::new().route("/", post(register_user).get(list_users));

    Router::new().nest("/users", routers)
}
