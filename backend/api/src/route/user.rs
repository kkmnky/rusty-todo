use axum::{
    Router,
    routing::{delete, get, post},
};
use registry::AppRegistry;

use crate::handler::user::{delete_user, get_current_user, list_users, register_user};

pub fn build_user_routers() -> Router<AppRegistry> {
    let routers = Router::new()
        .route("/", post(register_user).get(list_users))
        .route("/me", get(get_current_user))
        .route("/{user_id}", delete(delete_user));

    Router::new().nest("/users", routers)
}
