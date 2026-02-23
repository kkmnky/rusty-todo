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

fn build_app() -> axum::Router {
    let config = AppConfig::new().expect("DATABASE_* / REDIS_* / AUTH_* 環境変数が必要");
    let pool = connect_database_with(&config.database);
    let kv_store = Arc::new(RedisClient::new(&config.redis).expect("Redis接続が成功する"));
    let registry: AppRegistry = Arc::new(AppRegistryImpl::new(pool, kv_store, config));
    routes().with_state(registry)
}

async fn register_user_and_login(app: &axum::Router) -> (String, String) {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("timestamp")
        .as_nanos();
    let email = format!("integration-todo-{}@example.com", unique);
    let register_body = format!(
        r#"{{"name":"Integration Todo","email":"{}","password":"password123"}}"#,
        email
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(register_body))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let created: Value = serde_json::from_slice(&body).expect("json");
    let user_id = created["id"].as_str().expect("user_id").to_string();

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

    (user_id, auth_header)
}

async fn create_todo(app: &axum::Router, user_id: &str, auth_header: &str) -> String {
    let body = format!(
        r#"{{"title":"integration-todo","assigneeUserId":"{}"}}"#,
        user_id
    );
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/todos")
                .header(AUTHORIZATION, auth_header)
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
    created["id"].as_str().expect("todo_id").to_string()
}

async fn cleanup_user(app: &axum::Router, user_id: &str, auth_header: &str) {
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

#[tokio::test]
async fn todoのcrudが1シナリオで実行できる() {
    let app = build_app();
    let (user_id, auth_header) = register_user_and_login(&app).await;

    let todo_id = create_todo(&app, &user_id, &auth_header).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/todos/me")
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
    let todos: Value = serde_json::from_slice(&body).expect("json");
    let items = todos["items"].as_array().expect("items");
    assert!(items.iter().any(|item| item["id"] == todo_id));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/todos/{}", todo_id))
                .header(AUTHORIZATION, auth_header.clone())
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"integration-edited"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let edited: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(edited["id"], todo_id);
    assert_eq!(edited["title"], "integration-edited");
    assert_eq!(edited["assignee_user_id"], user_id);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/todos/{}/completed", todo_id))
                .header(AUTHORIZATION, auth_header.clone())
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"completed":true}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let completed: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(completed["id"], todo_id);
    assert_eq!(completed["completed"], true);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/todos/{}", todo_id))
                .header(AUTHORIZATION, auth_header.clone())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/todos/me")
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
    let todos: Value = serde_json::from_slice(&body).expect("json");
    let items = todos["items"].as_array().expect("items");
    assert!(!items.iter().any(|item| item["id"] == todo_id));

    cleanup_user(&app, &user_id, &auth_header).await;
}

#[tokio::test]
async fn completedにboolean以外を送ると400または422を返す() {
    let app = build_app();
    let (user_id, auth_header) = register_user_and_login(&app).await;
    let todo_id = create_todo(&app, &user_id, &auth_header).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/todos/{}/completed", todo_id))
                .header(AUTHORIZATION, auth_header.clone())
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"completed":"true"}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/todos/{}", todo_id))
                .header(AUTHORIZATION, auth_header.clone())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    cleanup_user(&app, &user_id, &auth_header).await;
}
