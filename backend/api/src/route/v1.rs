use axum::Router;
use registry::AppRegistry;

use crate::route::health::build_health_check_routers;

pub fn routes() -> Router<AppRegistry> {
    let routers = Router::new().merge(build_health_check_routers());
    Router::new().nest("/api/v1", routers)
}
