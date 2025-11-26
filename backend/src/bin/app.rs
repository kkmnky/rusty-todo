use anyhow::{Context, Result};
use api::route::v1;
use axum::{Router, routing::get};
use shared::env::{Environment, which};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    init_telemetry()?;

    let app = Router::new()
        .merge(v1::routes())
        .route("/", get(|| async { "Hello, World!" }));

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .await
        .context("Failed to bind to address")
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Failed to bind to address"
            );
        })
}

fn init_telemetry() -> Result<()> {
    let log_level = match which() {
        Environment::Development => "debug",
        Environment::Production => "info",
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());
    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .json();

    Registry::default()
        .with(subscriber)
        .with(env_filter)
        .try_init()?;

    Ok(())
}
