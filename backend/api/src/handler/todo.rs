use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use garde::Validate;
use kernel::{
    model::id::TodoId,
    usecase::todo::{
        list_my_todos::ListMyTodosUsecase,
        register::RegisterTodoUsecase,
        update_completed::{UpdateTodoCompletedInput, UpdateTodoCompletedUsecase},
    },
};
use registry::AppRegistry;
use shared::error::AppResult;

use crate::{
    handler::auth::require_auth,
    model::todo::{RegisterTodoRequest, TodoResponse, TodosResponse, UpdateTodoCompletedRequest},
};

pub async fn register_todo(
    State(registry): State<AppRegistry>,
    headers: HeaderMap,
    Json(req): Json<RegisterTodoRequest>,
) -> AppResult<(StatusCode, Json<TodoResponse>)> {
    require_auth(&registry, &headers)?;
    req.validate()?;

    let usecase = RegisterTodoUsecase::new(registry.todo_repository());
    let registered_user = usecase.execute(req.into()).await?;

    Ok((StatusCode::CREATED, Json(registered_user.into())))
}

pub async fn list_my_todos(
    State(registry): State<AppRegistry>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, Json<TodosResponse>)> {
    let verified_token = require_auth(&registry, &headers)?;
    let user_id = verified_token.sub;

    let usecase = ListMyTodosUsecase::new(registry.todo_repository());
    let items = usecase
        .execute(user_id)
        .await?
        .into_iter()
        .map(TodoResponse::from)
        .collect();

    Ok((StatusCode::OK, Json(TodosResponse { items })))
}

pub async fn update_todo_completed(
    State(registry): State<AppRegistry>,
    Path(todo_id): Path<TodoId>,
    headers: HeaderMap,
    Json(req): Json<UpdateTodoCompletedRequest>,
) -> AppResult<(StatusCode, Json<TodoResponse>)> {
    require_auth(&registry, &headers)?;

    let usecase = UpdateTodoCompletedUsecase::new(registry.todo_repository());
    let todo = usecase
        .execute(UpdateTodoCompletedInput {
            id: todo_id,
            completed: req.completed,
        })
        .await?;

    Ok((StatusCode::OK, Json(TodoResponse::from(todo))))
}

#[cfg(test)]
mod tests {
    use crate::handler::test_support::{
        build_auth_header, build_registry_with_auth_for_user, build_registry_with_jwt,
        build_registry_with_valid_auth,
    };
    use crate::model::todo::UpdateTodoCompletedRequest;
    use crate::route::todo::build_todo_routers;

    use super::*;
    use axum::{
        Json,
        body::Body,
        extract::{Path, State},
        http::{
            HeaderMap, Request, StatusCode,
            header::{AUTHORIZATION, CONTENT_TYPE},
        },
    };
    use kernel::{
        model::{
            id::{TodoId, UserId},
            todo::Todo,
        },
        repository::todo::{MockTodoRepository, TodoRepository},
        service::jwt::JwtIssuer,
    };
    use registry::{AppRegistry, MockAppRegistryExt};
    use rstest::rstest;
    use shared::error::AppError;
    use std::sync::Arc;
    use tower::util::ServiceExt;

    #[rstest]
    #[tokio::test]
    async fn タスク追加は201と必要項目を返す() {
        let (mut registry, headers) = build_registry_with_valid_auth();
        let assignee_user_id = UserId::new();
        let title = "買い物".to_string();

        let mut repo = MockTodoRepository::new();
        let title_for_match = title.clone();
        repo.expect_create()
            .withf(move |event| {
                event.title == title_for_match && event.assignee_user_id == assignee_user_id
            })
            .returning(move |event| {
                Ok(Todo {
                    id: TodoId::new(),
                    assignee_user_id: event.assignee_user_id,
                    title: event.title,
                    completed: false,
                    due_at: event.due_at,
                })
            });

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let req = RegisterTodoRequest::new(title, assignee_user_id, None);

        let result = register_todo(State(registry), headers, Json(req)).await;
        let (status, Json(body)) = result.expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body.assignee_user_id, assignee_user_id);
        assert_eq!(body.title, "買い物");
        assert!(!body.completed);
        assert!(body.due_at.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn タスク追加はtitle不正で失敗する() {
        let (registry, headers) = build_registry_with_valid_auth();
        let registry: AppRegistry = Arc::new(registry);
        let req = RegisterTodoRequest::new("".to_string(), UserId::new(), None);

        let err = register_todo(State(registry), headers, Json(req))
            .await
            .expect_err("バリデーションは失敗する");

        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn タスク追加はリポジトリ失敗でエラーになる() {
        let (mut registry, headers) = build_registry_with_valid_auth();
        let mut repo = MockTodoRepository::new();
        repo.expect_create()
            .returning(|_event| Err(AppError::SqlExecuteError(sqlx::Error::RowNotFound)));

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let req = RegisterTodoRequest::new("買い物".to_string(), UserId::new(), None);

        let err = register_todo(State(registry), headers, Json(req))
            .await
            .expect_err("リポジトリ失敗はエラーになる");

        assert!(matches!(err, AppError::SqlExecuteError(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn 自分のタスク一覧は200とitemsを返す() {
        let user_id = UserId::new();
        let (mut registry, headers) = build_registry_with_auth_for_user(user_id);
        let expected_todo_id = TodoId::new();
        let mut repo = MockTodoRepository::new();
        repo.expect_find_by_user_id()
            .withf(move |value| *value == user_id)
            .returning(move |assignee_user_id| {
                Ok(vec![Todo {
                    id: expected_todo_id,
                    assignee_user_id,
                    title: "自分のタスク".to_string(),
                    completed: false,
                    due_at: None,
                }])
            });

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);

        let (status, Json(body)) = list_my_todos(State(registry), headers)
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.items.len(), 1);
        assert_eq!(body.items[0].id, expected_todo_id);
        assert_eq!(body.items[0].assignee_user_id, user_id);
        assert_eq!(body.items[0].title, "自分のタスク");
        assert!(!body.items[0].completed);
        assert!(body.items[0].due_at.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn 自分のタスク一覧はjwtのsubで取得する() {
        let user_id = UserId::new();
        let (mut registry, headers) = build_registry_with_auth_for_user(user_id);
        let mut repo = MockTodoRepository::new();
        repo.expect_find_by_user_id()
            .withf(move |value| *value == user_id)
            .times(1)
            .returning(move |assignee_user_id| {
                Ok(vec![Todo {
                    id: TodoId::new(),
                    assignee_user_id,
                    title: "sub-user-todo".to_string(),
                    completed: false,
                    due_at: None,
                }])
            });

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);

        let (status, Json(body)) = list_my_todos(State(registry), headers)
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.items.len(), 1);
        assert_eq!(body.items[0].assignee_user_id, user_id);
    }

    #[rstest]
    #[tokio::test]
    async fn 自分のタスク一覧は0件でも200と空配列を返す() {
        let user_id = UserId::new();
        let (mut registry, headers) = build_registry_with_auth_for_user(user_id);
        let mut repo = MockTodoRepository::new();
        repo.expect_find_by_user_id()
            .withf(move |value| *value == user_id)
            .returning(|_| Ok(vec![]));

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);

        let (status, Json(body)) = list_my_todos(State(registry), headers)
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert!(body.items.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn 自分のタスク一覧はauthorizationヘッダがないと401を返す() {
        let mut repo = MockTodoRepository::new();
        repo.expect_find_by_user_id().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);
        let registry: AppRegistry = Arc::new(registry);
        let headers = HeaderMap::new();

        let err = list_my_todos(State(registry), headers)
            .await
            .expect_err("Authorizationヘッダなしは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn 自分のタスク一覧は不正jwtで401を返す() {
        let (mut registry, _jwt_issuer) = build_registry_with_jwt();
        let mut repo = MockTodoRepository::new();
        repo.expect_find_by_user_id().times(0);

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let wrong_issuer = JwtIssuer::new("wrong-secret".to_string(), 60_u64 * 60);
        let wrong_token = wrong_issuer.issue_token(UserId::new()).expect("jwt生成");
        let headers = build_auth_header(&wrong_token.0);

        let err = list_my_todos(State(registry), headers)
            .await
            .expect_err("不正JWTは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn タスク完了更新はcompleted_trueで200と更新後todoを返す() {
        let (mut registry, headers) = build_registry_with_valid_auth();
        let todo_id = TodoId::new();
        let assignee_user_id = UserId::new();
        let mut repo = MockTodoRepository::new();
        repo.expect_update_completed()
            .withf(move |event| event.id == todo_id && event.completed)
            .returning(move |event| {
                Ok(Todo {
                    id: event.id,
                    assignee_user_id,
                    title: "updated-todo".to_string(),
                    completed: event.completed,
                    due_at: None,
                })
            });

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let req = UpdateTodoCompletedRequest::new(true);

        let (status, Json(body)) =
            update_todo_completed(State(registry), Path(todo_id), headers, Json(req))
                .await
                .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.id, todo_id);
        assert_eq!(body.assignee_user_id, assignee_user_id);
        assert_eq!(body.title, "updated-todo");
        assert!(body.completed);
        assert!(body.due_at.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn タスク完了更新はcompleted_falseで200と更新後todoを返す() {
        let (mut registry, headers) = build_registry_with_valid_auth();
        let todo_id = TodoId::new();
        let assignee_user_id = UserId::new();
        let mut repo = MockTodoRepository::new();
        repo.expect_update_completed()
            .withf(move |event| event.id == todo_id && !event.completed)
            .returning(move |event| {
                Ok(Todo {
                    id: event.id,
                    assignee_user_id,
                    title: "updated-todo".to_string(),
                    completed: event.completed,
                    due_at: None,
                })
            });

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let req = UpdateTodoCompletedRequest::new(false);

        let (status, Json(body)) =
            update_todo_completed(State(registry), Path(todo_id), headers, Json(req))
                .await
                .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.id, todo_id);
        assert_eq!(body.assignee_user_id, assignee_user_id);
        assert_eq!(body.title, "updated-todo");
        assert!(!body.completed);
        assert!(body.due_at.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn タスク完了更新はauthorizationヘッダがないと401を返す() {
        let mut repo = MockTodoRepository::new();
        repo.expect_update_completed().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);
        let registry: AppRegistry = Arc::new(registry);
        let headers = HeaderMap::new();

        let err = update_todo_completed(
            State(registry),
            Path(TodoId::new()),
            headers,
            Json(UpdateTodoCompletedRequest::new(true)),
        )
        .await
        .expect_err("Authorizationヘッダなしは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn タスク完了更新は不正jwtで401を返す() {
        let (mut registry, _jwt_issuer) = build_registry_with_jwt();
        let mut repo = MockTodoRepository::new();
        repo.expect_update_completed().times(0);

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let wrong_issuer = JwtIssuer::new("wrong-secret".to_string(), 60_u64 * 60);
        let wrong_token = wrong_issuer.issue_token(UserId::new()).expect("jwt生成");
        let headers = build_auth_header(&wrong_token.0);

        let err = update_todo_completed(
            State(registry),
            Path(TodoId::new()),
            headers,
            Json(UpdateTodoCompletedRequest::new(true)),
        )
        .await
        .expect_err("不正JWTは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn タスク完了更新は不正なtodo_idで400を返す() {
        let (mut registry, headers) = build_registry_with_valid_auth();
        let mut repo = MockTodoRepository::new();
        repo.expect_update_completed().times(0);

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let app = build_todo_routers().with_state(registry);
        let auth_value = headers
            .get(AUTHORIZATION)
            .expect("Authorizationヘッダがある")
            .clone();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/todos/invalid-todo-id/completed")
                    .header(AUTHORIZATION, auth_value)
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"completed":true}"#))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[rstest]
    #[tokio::test]
    async fn タスク完了更新は存在しないtodo_idで404を返す() {
        let (mut registry, headers) = build_registry_with_valid_auth();
        let todo_id = TodoId::new();
        let mut repo = MockTodoRepository::new();
        repo.expect_update_completed()
            .withf(move |event| event.id == todo_id && event.completed)
            .returning(|_event| Err(AppError::EntityNotFoundError("todo not found".into())));

        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);

        let err = update_todo_completed(
            State(registry),
            Path(todo_id),
            headers,
            Json(UpdateTodoCompletedRequest::new(true)),
        )
        .await
        .expect_err("存在しないtodo_idは404を期待する");

        assert!(matches!(err, AppError::EntityNotFoundError(_)));
    }
}
