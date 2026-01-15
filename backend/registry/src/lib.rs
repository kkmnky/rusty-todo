use std::sync::Arc;

use adapter::{
    database::ConnectionPool,
    repository::{health::HealthCheckRepositoryImpl, user::UserRepositoryImpl},
};
use kernel::repository::{health::HealthCheckRepository, user::UserRepository};

#[derive(Clone)]
pub struct AppRegistryImpl {
    pub health_check_repository: Arc<dyn HealthCheckRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl AppRegistryImpl {
    pub fn new(pool: ConnectionPool) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(pool.clone()));
        let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));

        Self {
            health_check_repository,
            user_repository,
        }
    }

    pub fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    pub fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }
}

#[mockall::automock]
pub trait AppRegistryExt {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository>;
    fn user_repository(&self) -> Arc<dyn UserRepository>;
}

impl AppRegistryExt for AppRegistryImpl {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }
}

pub type AppRegistry = Arc<dyn AppRegistryExt + Send + Sync>;
