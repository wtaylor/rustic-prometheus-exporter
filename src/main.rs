use std::path::PathBuf;

use axum::{Router, extract::State, routing::get};
use clap::{Parser, Subcommand};
use config::{Config, File};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tokio::{signal, task};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::options::AppOptions;

mod collector_scheduler;
mod options;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a config file can be absolute or relative
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    /// Subcommand to run, use -h to see available commands
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// starts the exporter
    Run {},
}

#[derive(Clone)]
struct AppState {
    metrics_handle: PrometheusHandle,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let shutdown_token = CancellationToken::new();

    let cli = Cli::parse();

    info!("starting up exporter");

    let settings_monitor = Config::builder()
        .add_source(File::from(cli.config))
        .add_source(config::Environment::with_prefix("RPE"))
        .build()
        .unwrap();

    let settings = settings_monitor.try_deserialize::<AppOptions>().unwrap();

    let metrics_handle = PrometheusBuilder::new().install_recorder().unwrap();
    let state = AppState { metrics_handle };

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(settings.http.listen)
        .await
        .unwrap();

    info!("listening on {}", settings.http.listen);
    info!(
        "metrics endpoint is available at http://{}/metrics",
        settings.http.listen
    );

    let collector_worker_task =
        task::spawn(collector_scheduler::start(settings, shutdown_token.clone()));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_token.clone()))
        .await
        .unwrap();
}

async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}

async fn shutdown_signal(cancellation_token: CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received, shutting down");
    cancellation_token.cancel();
}
