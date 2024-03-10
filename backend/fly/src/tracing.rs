use std::collections::HashMap;

use opentelemetry::{global, KeyValue};

use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry_semantic_conventions as semcov;
use shared::env::{load_public, load_secret};
use tracing::{debug, info, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};

pub fn init_opentelemetry_from_environment() -> Result<(), Box<dyn std::error::Error>> {
    let honeycomb_api_key = load_secret("HONEYCOMB_API_KEY")?;
    let tracing_exporter_http_endpoint = load_public("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")?;
    debug!("using '{}' as endpoint", tracing_exporter_http_endpoint);

    let headers = HashMap::from([("x-honeycomb-team".into(), honeycomb_api_key.into())]);
    let resource = Resource::new([KeyValue::new(semcov::resource::SERVICE_NAME, "fosdem2024")]);
    let resource = if let Ok(region) = dotenvy::var("FLY_REGION") {
        resource.merge(&Resource::new([KeyValue::new(
            semcov::resource::CLOUD_REGION,
            region,
        )]))
    } else {
        resource
    };
    let resource = if let Ok(env) = dotenvy::var("OTEL_DEPLOYMENT_ENVIRONMENT") {
        resource.merge(&Resource::new([KeyValue::new(
            semcov::resource::DEPLOYMENT_ENVIRONMENT,
            env,
        )]))
    } else {
        resource
    };

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(tracing_exporter_http_endpoint)
                .with_headers(headers),
        )
        .with_trace_config(
            sdktrace::config()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        )
        .install_batch(runtime::Tokio)?;

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let subscriber = Registry::default()
        .with(telemetry_layer)
        .with(filter_layer)
        .with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)?;
    let root = span!(tracing::Level::TRACE, "init_from_environment");
    let _enter = root.enter();
    info!("tracing initialised globally");

    Ok(())
}

pub fn init_safe_default_from_environment() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    Ok(())
}
