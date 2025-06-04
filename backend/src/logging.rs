use std::{fs::OpenOptions, io::Write};
use tracing::info;
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tower_http::trace::TraceLayer;
use tower_http::classify::{SharedClassifier, ServerErrorsAsFailures};

pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Create log file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("linksphere_logs.md")?;

    // Write markdown header if file is empty
    if file.metadata()?.len() == 0 {
        writeln!(file, "# LinkSphere Server Logs\n")?;
        writeln!(file, "| Timestamp | Level | Event | Details |")?;
        writeln!(file, "|-----------|--------|--------|----------|")?;
    }

    // Setup console logging format
    let console_layer = fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_timer(UtcTime::rfc_3339())
        .compact();

    // Create a custom env filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("info,backend=debug,tower_http=debug")
        });

    // Initialize the tracing subscriber with console layer
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .init();

    info!("🔍 Logging system initialized");
    Ok(())
}

pub fn create_trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}
