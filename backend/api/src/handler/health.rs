use axum::{extract::State, http::StatusCode};
use registry::AppRegistry;

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn health_check_db(State(registry): State<AppRegistry>) -> StatusCode {
    if registry.health_check_repository().check_db().await {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use kernel::repository::health::{HealthCheckRepository, MockHealthCheckRepository};
    use registry::MockAppRegistryExt;
    use std::sync::Arc;

    #[tokio::test]
    async fn ヘルスチェックは200を返す() {
        assert_eq!(health_check().await, StatusCode::OK);
    }

    #[tokio::test]
    async fn dbヘルスチェック成功なら200() {
        let mut repo = MockHealthCheckRepository::new();
        repo.expect_check_db().returning(|| true);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn HealthCheckRepository> = Arc::new(repo);
        registry
            .expect_health_check_repository()
            .return_const(repo_arc.clone());

        let registry: AppRegistry = Arc::new(registry);

        let status = health_check_db(State(registry)).await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn dbヘルスチェック失敗なら500() {
        let mut repo = MockHealthCheckRepository::new();
        repo.expect_check_db().returning(|| false);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn HealthCheckRepository> = Arc::new(repo);
        registry
            .expect_health_check_repository()
            .return_const(repo_arc.clone());

        let registry: AppRegistry = Arc::new(registry);

        let status = health_check_db(State(registry)).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
