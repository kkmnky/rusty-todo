use axum::{Router, routing::post};
use registry::AppRegistry;

use crate::handler::auth::auth_login;

pub fn build_auth_routers() -> Router<AppRegistry> {
    let routers = Router::new().route("/login", post(auth_login));

    Router::new().nest("/auth", routers)
}
