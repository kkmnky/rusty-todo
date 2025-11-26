use axum::{Router, routing::get};

use crate::handler::health::health_check;

pub fn build_health_check_routers() -> Router<()> {
    let routers = Router::new().route("/", get(health_check));

    Router::new().nest("/health", routers)
}
