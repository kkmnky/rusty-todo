use axum::{
    Router,
    routing::{delete, post},
};
use registry::AppRegistry;

use crate::handler::user::{delete_user, list_users, register_user};

pub fn build_user_routers() -> Router<AppRegistry> {
    let routers = Router::new()
        .route("/", post(register_user).get(list_users))
        .route("/:user_id", delete(delete_user));

    Router::new().nest("/users", routers)
}
