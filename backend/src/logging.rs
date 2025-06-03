use std::{time::Duration, fs::OpenOptions, io::Write};
use tracing::{Level, info, warn, error, Subscriber};
use tracing_subscriber::{
    fmt::{self, time::UtcTime, writer::MakeWriterExt},
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse, DefaultOnRequest};
use axum::http::Request;
use uuid::Uuid;
use std::sync::Mutex;
use chrono::Utc;

// Custom writer for markdown formatting
#[derive(Clone)]
struct MarkdownWriter {
    file: std::sync::Arc<Mutex<std::fs::File>>,
}

impl MarkdownWriter {
    fn new(file_path: &str) -> std::io::Result<Self> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        
        // Write markdown header if file is empty
        if file.metadata()?.len() == 0 {
            writeln!(file, "# LinkSphere Server Logs\n")?;
            writeln!(file, "| Timestamp | Level | Event | Details |")?;
            writeln!(file, "|-----------|--------|--------|----------|")?;
        }

        Ok(Self {
            file: std::sync::Arc::new(Mutex::new(file)),
        })
    }
}

impl std::io::Write for MarkdownWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = String::from_utf8(buf.to_vec()) {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            
            // Parse the log message and format it as a markdown table row
            let formatted = if s.trim().is_empty() {
                String::new()
            } else {
                let level = if s.contains("ERROR") {
                    "❌ ERROR"
                } else if s.contains("WARN") {
                    "⚠️ WARN"
                } else {
                    "ℹ️ INFO"
                };

                let parts: Vec<&str> = s.splitn(2, ' ').collect();
                let details = parts.get(1).unwrap_or(&s).trim();
                
                format!("| {} | {} | {} |\n", now, level, details)
            };

            if !formatted.is_empty() {
                let mut file = self.file.lock().unwrap();
                file.write_all(formatted.as_bytes())?;
                file.flush()?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.lock().unwrap().flush()
    }
}

pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Setup console logging format
    let console_layer = fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_timer(UtcTime::rfc_3339())
        .compact();

    // Setup file logging with markdown format
    let file_writer = MarkdownWriter::new("linksphere_logs.md")?;
    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .with_file(false)
        .with_timer(UtcTime::rfc_3339())
        .compact();

    // Create a custom env filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("info,backend=debug,tower_http=debug")
        });

    // Initialize the tracing subscriber with both console and file layers
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("🔍 Logging system initialized");
    Ok(())
}

pub fn create_trace_layer() -> TraceLayer<SharedState> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_request(|request: &Request<_>, _span: &tracing::Span| {
            // Generate a unique request ID
            let request_id = Uuid::new_v4();
            
            info!(
                "➡️ Request: {} {} [ID: {}]",
                request.method(),
                request.uri(),
                request_id
            );

            // Log headers if they exist
            if !request.headers().is_empty() {
                info!("Headers: {:?}", request.headers());
            }
        })
        .on_response(|response, latency: Duration, _span: &tracing::Span| {
            let status = response.status();
            let latency_ms = latency.as_secs_f64() * 1000.0;

            if status.is_success() {
                info!(
                    "✅ Response: {} - {:.2}ms",
                    status,
                    latency_ms
                );
            } else if status.is_client_error() {
                warn!(
                    "⚠️ Client Error: {} - {:.2}ms",
                    status,
                    latency_ms
                );
            } else if status.is_server_error() {
                error!(
                    "❌ Server Error: {} - {:.2}ms",
                    status,
                    latency_ms
                );
            }
        })
}

// Custom type for sharing state between request and response
#[derive(Clone, Debug)]
pub struct SharedState; 