use shared::{
    config::DatabaseConfig,
    error::{AppError, AppResult},
};
use sqlx::{PgPool, Postgres, postgres::PgConnectOptions};
use sqlx_tracing::Pool as TracingPool;
use std::sync::Arc;

pub mod model;

#[derive(Clone)]
pub struct ConnectionPool {
    raw: PgPool,
    traced: Arc<TracingPool<Postgres>>,
}

impl ConnectionPool {
    pub fn new(pool: PgPool) -> Self {
        Self {
            raw: pool.clone(),
            traced: Arc::new(TracingPool::from(pool.clone())),
        }
    }

    pub fn inner_ref(&self) -> &TracingPool<Postgres> {
        self.traced.as_ref()
    }

    pub async fn begin(&self) -> AppResult<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.raw.begin().await.map_err(AppError::TransactionError)
    }
}

pub fn connect_database_with(cfg: &DatabaseConfig) -> ConnectionPool {
    ConnectionPool::new(PgPool::connect_lazy_with(make_pg_connect_options(cfg)))
}

fn make_pg_connect_options(cfg: &DatabaseConfig) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.username)
        .password(&cfg.password)
        .database(&cfg.database)
}
