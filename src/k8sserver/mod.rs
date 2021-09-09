use crate::manager::Manager;
use crate::Result;
use actix_web::{
    get, middleware,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};
mod health;
mod metrics;

#[actix_rt::main]
pub async fn start_server(addr: String) -> Result<()> {
    #[cfg(feature = "telemetry")]
    let otlp_endpoint = std::env::var("OPENTELEMETRY_ENDPOINT_URL")
        .expect("Need a otel tracing collector configured");

    #[cfg(feature = "telemetry")]
    let tracer = opentelemetry_otlp::new_pipeline()
        .with_endpoint(&otlp_endpoint)
        .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
            opentelemetry::sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                "service.name",
                "foo-controller",
            )]),
        ))
        .with_tonic()
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap();

    // Finish layers
    #[cfg(feature = "telemetry")]
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let logger = tracing_subscriber::fmt::layer().json();

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    // Register all subscribers
    #[cfg(feature = "telemetry")]
    let collector = Registry::default()
        .with(telemetry)
        .with(logger)
        .with(env_filter);
    #[cfg(not(feature = "telemetry"))]
    let collector = Registry::default().with(logger).with(env_filter);

    tracing::subscriber::set_global_default(collector).unwrap();

    // Start kubernetes controller
    let (manager, drainer) = Manager::new().await;

    // Start web server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(manager.clone()))
            .wrap(middleware::Logger::default().exclude("/health"))
            .service(health::health)
            .service(metrics::metrics)
    })
    .bind(addr)
    .expect("Can not bind")
    .shutdown_timeout(0);

    tokio::select! {
        _ = drainer => warn!("controller drained"),
        _ = server.run() => info!("actix exited"),
    }
    Ok(())
}
