use anyhow::Result;
use opentelemetry::{KeyValue, global::set_tracer_provider, trace::TracerProvider};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, registry, util::SubscriberInitExt};

use crate::cross_cutting::config::{ObservabilityConfig, PackageConfig};

pub fn init_observability(
    ObservabilityConfig { endpoint, level }: ObservabilityConfig,
    PackageConfig {
        package_name,
        workspace_name,
        version,
    }: PackageConfig,
) -> Result<()> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .build()?;
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new(
                "service.name",
                format!("{}/{}", workspace_name, package_name),
            ),
            KeyValue::new("service.version", version),
        ])
        .build();
    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();
    let tracer = provider.tracer(workspace_name);

    set_tracer_provider(provider);

    registry()
        .with(EnvFilter::new(level))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    Ok(())
}
