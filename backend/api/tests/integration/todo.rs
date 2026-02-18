use std::sync::Arc;

use adapter::{database::connect_database_with, redis::RedisClient};
use api::route::v1::routes;
use axum::body::Body;
use axum::http::{
    Request, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use kernel::model::id::UserId;
use registry::{AppRegistry, AppRegistryImpl};
use shared::config::AppConfig;
use tower::util::ServiceExt;

#[tokio::test]
async fn todo追加はauthorizationヘッダがないと401を返す() {
    let config = AppConfig::new().expect("DATABASE_* / REDIS_* / AUTH_* 環境変数が必要");
    let pool = connect_database_with(&config.database);
    let kv_store = Arc::new(RedisClient::new(&config.redis).expect("Redis接続が成功する"));
    let registry: AppRegistry = Arc::new(AppRegistryImpl::new(pool, kv_store, config));
    let app = routes().with_state(registry);

    let body = format!(
        r#"{{"title":"integration-todo","assigneeUserId":"{}","dueAt":null}}"#,
        UserId::new()
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/todos")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn todo追加は不正jwtで401を返す() {
    let config = AppConfig::new().expect("DATABASE_* / REDIS_* / AUTH_* 環境変数が必要");
    let pool = connect_database_with(&config.database);
    let kv_store = Arc::new(RedisClient::new(&config.redis).expect("Redis接続が成功する"));
    let registry: AppRegistry = Arc::new(AppRegistryImpl::new(pool, kv_store, config));
    let app = routes().with_state(registry);

    let body = format!(
        r#"{{"title":"integration-todo","assigneeUserId":"{}","dueAt":null}}"#,
        UserId::new()
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/todos")
                .header(CONTENT_TYPE, "application/json")
                .header(AUTHORIZATION, "Bearer invalid-token")
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
