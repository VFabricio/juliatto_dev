use anyhow::Result;
use opentelemetry::{KeyValue, global::set_tracer_provider, trace::TracerProvider};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, registry, util::SubscriberInitExt};

use crate::cross_cutting::config::ObservabilityConfig;

pub fn init_observability(config: ObservabilityConfig) -> Result<()> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(config.endpoint.clone())
        .build()?;
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
        ])
        .build();
    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();
    let tracer = provider.tracer("");

    set_tracer_provider(provider);

    registry()
        .with(EnvFilter::new(config.level))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    Ok(())
}
