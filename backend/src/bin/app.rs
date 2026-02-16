use adapter::{database::connect_database_with, redis::RedisClient};
use anyhow::{Context, Result};
use api::route::v1;
use axum::{
    Router,
    body::Body,
    extract::{MatchedPath, Request},
    http::HeaderValue,
    routing::get,
};
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
    time::Duration,
};
use tokio::net::TcpListener;
use tower_http::{
    classify::ServerErrorsFailureClass,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::Level;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry = init_telemetry()?;

    let app_config = AppConfig::new()?;

    let pool = connect_database_with(&app_config.database);
    let kv_store = Arc::new(RedisClient::new(&app_config.redis)?);
    let registry = Arc::new(AppRegistryImpl::new(pool, kv_store, app_config));

    let app = with_request_trace_layers(
        Router::new()
            .merge(v1::routes())
            .route("/", get(|| async { "Hello, World!" })),
    )
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

#[derive(Clone, Copy, Default)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, request: &Request<B>) -> Option<RequestId> {
        if let Some(value) = request.headers().get("x-request-id") {
            return Some(RequestId::new(value.clone()));
        }

        let value = HeaderValue::from_str(&Uuid::new_v4().to_string()).ok()?;
        Some(RequestId::new(value))
    }
}

fn with_request_trace_layers<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<Body>| {
            let path = request
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str)
                .unwrap_or_else(|| request.uri().path());
            let span_name = format!("{} {}", request.method(), path);
            tracing::span!(
                Level::INFO,
                "http.request",
                otel.name = %span_name,
                request_id = tracing::field::Empty,
                method = %request.method(),
                path = %path,
                user_id = tracing::field::Empty,
            )
        })
        .on_request(
            |request: &axum::http::Request<Body>, span: &tracing::Span| {
                if let Some(request_id) = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                {
                    span.record("request_id", request_id);
                }
                tracing::debug!(parent: span, event.name = "request.received", "request received");
            },
        )
        .on_response(
            |response: &axum::http::Response<_>, latency: Duration, span: &tracing::Span| {
                let status = response.status().as_u16();
                let latency_ms = latency.as_millis();
                if response.status().is_server_error() {
                    tracing::error!(
                        parent: span,
                        event.name = "request.completed",
                        status,
                        latency_ms,
                        "request completed"
                    );
                } else if response.status().is_client_error() {
                    tracing::warn!(
                        parent: span,
                        event.name = "request.completed",
                        status,
                        latency_ms,
                        "request completed"
                    );
                } else {
                    tracing::info!(
                        parent: span,
                        event.name = "request.completed",
                        status,
                        latency_ms,
                        "request completed"
                    );
                }
            },
        )
        .on_failure(
            |failure: ServerErrorsFailureClass, latency: Duration, span: &tracing::Span| {
                tracing::error!(
                    parent: span,
                    event.name = "request.failure",
                    error.kind = ?failure,
                    latency_ms = latency.as_millis(),
                    "request failure"
                );
            },
        );

    router
        .route_layer(trace_layer)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}

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

    let env_filter = build_env_filter(log_level)?;
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

fn build_env_filter(default_level: &str) -> Result<EnvFilter> {
    let base = EnvFilter::try_from_default_env().unwrap_or_else(|_| default_level.into());

    let suppress_opentelemetry_sdk_debug = "opentelemetry_sdk=info"
        .parse()
        .context("Failed to parse opentelemetry_sdk log directive")?;
    let suppress_opentelemetry_debug = "opentelemetry=info"
        .parse()
        .context("Failed to parse opentelemetry log directive")?;
    let suppress_span_processor_debug = "opentelemetry_sdk::trace::span_processor=info"
        .parse()
        .context("Failed to parse opentelemetry_sdk span processor log directive")?;

    Ok(base
        .add_directive(suppress_opentelemetry_sdk_debug)
        .add_directive(suppress_opentelemetry_debug)
        .add_directive(suppress_span_processor_debug))
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
