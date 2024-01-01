// use std::env;

// use opentelemetry::KeyValue;
// use opentelemetry_otlp::WithExportConfig;
// use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
// use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

// pub fn init_tracing(service_name: &str) -> anyhow::Result<()> {
//     env::set_var("OTEL_BSP_MAX_EXPORT_BATCH_SIZE", "12");

//     let tracer =
//         opentelemetry_otlp::new_pipeline()
//             .tracing()
//             .with_exporter(
//                 opentelemetry_otlp::new_exporter()
//                     .tonic()
//                     .with_endpoint("http://localhost:4317"),
//             )
//             .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
//                 KeyValue::new("service.name", service_name.to_owned()),
//             ])))
//             .install_batch(runtime::Tokio)?;

//     tracing_subscriber::registry()
//         .with(tracing_subscriber::filter::EnvFilter::from_default_env())
//         .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
//         .with(tracing_opentelemetry::layer().with_tracer(tracer))
//         .try_init()?;

//     Ok(())
// }
