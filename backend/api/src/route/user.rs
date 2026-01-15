use axum::{Router, routing::post};
use registry::AppRegistry;

use crate::handler::user::register_user;

pub fn build_user_routers() -> Router<AppRegistry> {
    let routers = Router::new().route("/", post(register_user));

    Router::new().nest("/users", routers)
}
