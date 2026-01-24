use std::sync::Arc;

use adapter::{
    database::ConnectionPool,
    redis::RedisClient,
    repository::{
        auth::AuthRepositoryImpl, health::HealthCheckRepositoryImpl, user::UserRepositoryImpl,
    },
};
use kernel::repository::{
    auth::AuthRepository, health::HealthCheckRepository, user::UserRepository,
};
use shared::config::AppConfig;

#[derive(Clone)]
pub struct AppRegistryImpl {
    pub health_check_repository: Arc<dyn HealthCheckRepository>,
    pub user_repository: Arc<dyn UserRepository>,
    pub auth_repository: Arc<dyn AuthRepository>,
}

impl AppRegistryImpl {
    pub fn new(pool: ConnectionPool, kv_store: Arc<RedisClient>, app_config: AppConfig) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(pool.clone()));
        let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
        let auth_repository = Arc::new(AuthRepositoryImpl::new(
            pool.clone(),
            kv_store,
            app_config.auth.ttl,
        ));

        Self {
            health_check_repository,
            user_repository,
            auth_repository,
        }
    }

    pub fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    pub fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }

    pub fn auth_repository(&self) -> Arc<dyn AuthRepository> {
        self.auth_repository.clone()
    }
}

#[mockall::automock]
pub trait AppRegistryExt {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository>;
    fn user_repository(&self) -> Arc<dyn UserRepository>;
    fn auth_repository(&self) -> Arc<dyn AuthRepository>;
}

impl AppRegistryExt for AppRegistryImpl {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }

    fn auth_repository(&self) -> Arc<dyn AuthRepository> {
        self.auth_repository.clone()
    }
}

pub type AppRegistry = Arc<dyn AppRegistryExt + Send + Sync>;
