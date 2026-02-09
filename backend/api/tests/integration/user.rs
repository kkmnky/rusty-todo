use std::sync::Arc;

use adapter::{database::connect_database_with, redis::RedisClient};
use api::route::v1::routes;
use axum::body::{Body, to_bytes};
use axum::http::{
    Request, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use registry::{AppRegistry, AppRegistryImpl};
use serde_json::Value;
use shared::config::AppConfig;
use tower::util::ServiceExt;

#[tokio::test]
async fn ユーザの作成参照更新削除ができる() {
    let config = AppConfig::new().expect("DATABASE_* / REDIS_* / AUTH_* 環境変数が必要");
    let pool = connect_database_with(&config.database);
    let kv_store = Arc::new(RedisClient::new(&config.redis).expect("Redis接続が成功する"));
    let registry: AppRegistry = Arc::new(AppRegistryImpl::new(pool, kv_store, config));
    let app = routes().with_state(registry);

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("timestamp")
        .as_nanos();
    let email = format!("integration-{}@example.com", unique);
    let body = format!(
        r#"{{"name":"Integration","email":"{}","password":"password123"}}"#,
        email
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let created: Value = serde_json::from_slice(&body).expect("json");
    let user_id = created["id"].as_str().expect("user_id");

    let login_body = format!(r#"{{"email":"{}","password":"password123"}}"#, email);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(login_body))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let login: Value = serde_json::from_slice(&body).expect("json");
    let access_token = login["access_token"].as_str().expect("access_token");
    let auth_header = format!("Bearer {}", access_token);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .header(AUTHORIZATION, auth_header.clone())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let users: Value = serde_json::from_slice(&body).expect("json");
    let items = users["items"].as_array().expect("items");
    assert!(items.iter().any(|item| item["email"] == email));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/me")
                .header(AUTHORIZATION, auth_header.clone())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let update_body = r#"{"currentPassword":"password123","newPassword":"password456"}"#;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/users/me/password")
                .header(AUTHORIZATION, auth_header.clone())
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(update_body))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let relogin_body = format!(r#"{{"email":"{}","password":"password456"}}"#, email);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(relogin_body))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/users/{}", user_id))
                .header(AUTHORIZATION, auth_header)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
