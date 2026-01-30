use axum::Router;
use registry::AppRegistry;

use crate::route::{
    auth::build_auth_routers, health::build_health_check_routers, user::build_user_routers,
};

pub fn routes() -> Router<AppRegistry> {
    let routers = Router::new()
        .merge(build_health_check_routers())
        .merge(build_auth_routers())
        .merge(build_user_routers());
    Router::new().nest("/api/v1", routers)
}
