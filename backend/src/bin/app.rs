use adapter::{database::connect_database_with, redis::RedisClient};
use anyhow::{Context, Result};
use api::route::v1;
use axum::{Router, routing::get};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use registry::AppRegistryImpl;
use shared::{
    config::AppConfig,
    env::{Environment, which},
};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry = init_telemetry()?;

    let app_config = AppConfig::new()?;

    let pool = connect_database_with(&app_config.database);
    let kv_store = Arc::new(RedisClient::new(&app_config.redis)?);
    let registry = Arc::new(AppRegistryImpl::new(pool, kv_store, app_config));

    let app = Router::new()
        .merge(v1::routes())
        .route("/", get(|| async { "Hello, World!" }))
        .with_state(registry);

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

struct TelemetryGuard;


fn init_telemetry() -> Result<TelemetryGuard> {
    let env = which();

    let log_level = match env {
        Environment::Development => "debug",
        Environment::Production => "info",
    };
    let env_name = match env {
        Environment::Development => "dev",
        Environment::Production => "prod",
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());
    let log_format = select_log_format(&env);

    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());
    global::set_text_map_propagator(TraceContextPropagator::new());

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .expect("Failed to build the span exporter");

    let resource = Resource::builder()
        .with_service_name("rusty-todo")
        .with_attribute(KeyValue::new("deployment.environment", env_name))
        .build();

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(otlp_exporter)
        .build();
    match log_format {
        LogFormat::Json => {
            let tracer = tracer_provider.tracer("rusty-todo");
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_file(true)
                .with_line_number(true)
                .with_target(false);
            Registry::default()
                .with(fmt_layer)
                .with(env_filter)
                .with(otel_layer)
                .try_init()?;
        }
        LogFormat::Pretty => {
            let tracer = tracer_provider.tracer("rusty-todo");
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
            let fmt_layer = tracing_subscriber::fmt::layer()
                .pretty()
                .with_file(true)
                .with_line_number(true)
                .with_target(false);
            Registry::default()
                .with(fmt_layer)
                .with(env_filter)
                .with(otel_layer)
                .try_init()?;
        }
    }

    global::set_tracer_provider(tracer_provider);

    Ok(TelemetryGuard)
}

#[derive(Clone, Copy)]
enum LogFormat {
    Json,
    Pretty,
}

fn select_log_format(env: &Environment) -> LogFormat {
    let default_format = match env {
        Environment::Development => "pretty",
        Environment::Production => "json",
    };
    match std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| default_format.to_string())
        .as_str()
    {
        "pretty" => LogFormat::Pretty,
        "json" => LogFormat::Json,
        _ => match env {
            Environment::Development => LogFormat::Pretty,
            Environment::Production => LogFormat::Json,
        },
    }
}
