use std::sync::Arc;

use adapter::{database::ConnectionPool, repository::health::HealthCheckRepositoryImpl};
use kernel::repository::health::HealthCheckRepository;

#[derive(Clone)]
pub struct AppRegistryImpl {
    pub health_check_repository: Arc<dyn HealthCheckRepository>,
}

impl AppRegistryImpl {
    pub fn new(pool: ConnectionPool) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(pool));
        Self {
            health_check_repository,
        }
    }

    pub fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }
}

#[mockall::automock]
pub trait AppRegistryExt {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository>;
}

impl AppRegistryExt for AppRegistryImpl {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }
}

pub type AppRegistry = Arc<dyn AppRegistryExt + Send + Sync>;
