use axum::Router;

use crate::route::health::build_health_check_routers;

pub fn routes() -> Router<()> {
    let routers = Router::new().merge(build_health_check_routers());
    Router::new().nest("/api/v1", routers)
}
