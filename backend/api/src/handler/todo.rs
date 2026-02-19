use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use garde::Validate;
use kernel::usecase::todo::{list_my_todos::ListMyTodosUsecase, register::RegisterTodoUsecase};
use registry::AppRegistry;
use shared::error::AppResult;

use crate::{
    handler::auth::require_auth,
    model::todo::{RegisterTodoRequest, TodoResponse, TodosResponse},
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

#[cfg(test)]
mod tests {
    use crate::handler::test_support::{
        build_auth_header, build_auth_header_for_user, build_test_jwt_issuer,
        build_valid_auth_header,
    };

    use super::*;
    use axum::{
        Json,
        extract::State,
        http::{HeaderMap, StatusCode},
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
    use rstest::{fixture, rstest};
    use shared::error::AppError;
    use std::sync::Arc;

    #[fixture]
    fn jwt_issuer() -> Arc<JwtIssuer> {
        build_test_jwt_issuer()
    }

    #[rstest]
    #[tokio::test]
    async fn タスク追加は201と必要項目を返す(jwt_issuer: Arc<JwtIssuer>) {
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

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let req = RegisterTodoRequest::new(title, assignee_user_id, None);
        let headers = build_valid_auth_header(&jwt_issuer);

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
    async fn タスク追加はtitle不正で失敗する(jwt_issuer: Arc<JwtIssuer>) {
        let mut registry = MockAppRegistryExt::new();
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());
        let registry: AppRegistry = Arc::new(registry);
        let req = RegisterTodoRequest::new("".to_string(), UserId::new(), None);
        let headers = build_valid_auth_header(&jwt_issuer);

        let err = register_todo(State(registry), headers, Json(req))
            .await
            .expect_err("バリデーションは失敗する");

        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn タスク追加はリポジトリ失敗でエラーになる(
        jwt_issuer: Arc<JwtIssuer>,
    ) {
        let mut repo = MockTodoRepository::new();
        repo.expect_create()
            .returning(|_event| Err(AppError::SqlExecuteError(sqlx::Error::RowNotFound)));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let req = RegisterTodoRequest::new("買い物".to_string(), UserId::new(), None);
        let headers = build_valid_auth_header(&jwt_issuer);

        let err = register_todo(State(registry), headers, Json(req))
            .await
            .expect_err("リポジトリ失敗はエラーになる");

        assert!(matches!(err, AppError::SqlExecuteError(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn 自分のタスク一覧は200とitemsを返す(jwt_issuer: Arc<JwtIssuer>) {
        let user_id = UserId::new();
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

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_auth_header_for_user(&jwt_issuer, user_id);

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
    async fn 自分のタスク一覧はjwtのsubで取得する(jwt_issuer: Arc<JwtIssuer>) {
        let user_id = UserId::new();
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

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_auth_header_for_user(&jwt_issuer, user_id);

        let (status, Json(body)) = list_my_todos(State(registry), headers)
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.items.len(), 1);
        assert_eq!(body.items[0].assignee_user_id, user_id);
    }

    #[rstest]
    #[tokio::test]
    async fn 自分のタスク一覧は0件でも200と空配列を返す(
        jwt_issuer: Arc<JwtIssuer>,
    ) {
        let user_id = UserId::new();
        let mut repo = MockTodoRepository::new();
        repo.expect_find_by_user_id()
            .withf(move |value| *value == user_id)
            .returning(|_| Ok(vec![]));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_auth_header_for_user(&jwt_issuer, user_id);

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
    async fn 自分のタスク一覧は不正jwtで401を返す(jwt_issuer: Arc<JwtIssuer>) {
        let mut repo = MockTodoRepository::new();
        repo.expect_find_by_user_id().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn TodoRepository> = Arc::new(repo);
        registry.expect_todo_repository().return_const(repo_arc);
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let wrong_issuer = JwtIssuer::new("wrong-secret".to_string(), 60_u64 * 60);
        let wrong_token = wrong_issuer.issue_token(UserId::new()).expect("jwt生成");
        let headers = build_auth_header(&wrong_token.0);

        let err = list_my_todos(State(registry), headers)
            .await
            .expect_err("不正JWTは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}
