use shared::config::AppConfig;
use sqlx::{PgPool, postgres::PgConnectOptions};

pub struct ConnectionPool(PgPool);

impl ConnectionPool {
    pub fn inner_ref(&self) -> &PgPool {
        &self.0
    }
}

pub fn connect_database_with(cfg: &AppConfig) -> ConnectionPool {
    ConnectionPool(PgPool::connect_lazy_with(make_pg_connect_options(cfg)))
}

fn make_pg_connect_options(cfg: &AppConfig) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&cfg.database.host)
        .port(cfg.database.port)
        .username(&cfg.database.username)
        .password(&cfg.database.password)
        .database(&cfg.database.database)
}
