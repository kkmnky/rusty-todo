use axum::{Router, routing::post};
use registry::AppRegistry;

use crate::handler::auth::{auth_login, auth_logout};

pub fn build_auth_routers() -> Router<AppRegistry> {
    let routers = Router::new()
        .route("/login", post(auth_login))
        .route("/logout", post(auth_logout));

    Router::new().nest("/auth", routers)
}
