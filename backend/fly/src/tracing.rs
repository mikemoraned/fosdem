use std::collections::HashMap;

use opentelemetry::{global, KeyValue};

use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry_semantic_conventions as semcov;
use shared::env::load_secret;
use tracing::{debug, info, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

pub fn init_from_environment() -> Result<(), Box<dyn std::error::Error>> {
    let honeycomb_api_key = load_secret("HONEYCOMB_API_KEY");
    let tracing_exporter_http_endpoint = dotenvy::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")?;
    debug!("using '{}' as endpoint", tracing_exporter_http_endpoint);

    let headers = HashMap::from([("x-honeycomb-team".into(), honeycomb_api_key.into())]);
    let resource = Resource::new([KeyValue::new(semcov::resource::SERVICE_NAME, "fosdem2024")]);

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

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(telemetry);

    tracing::subscriber::set_global_default(subscriber)?;
    let root = span!(tracing::Level::TRACE, "init_from_environment");
    let _enter = root.enter();
    info!("tracing initialised globally");

    Ok(())
}
