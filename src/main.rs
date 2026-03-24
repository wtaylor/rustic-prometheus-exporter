use std::path::PathBuf;

use axum::{Router, extract::State, routing::get};
use clap::{Parser, Subcommand};
use config::{Config, File};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

use crate::options::AppOptions;

mod options;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a config file can be absolute or relative
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    /// Subcommant to run, use -h to see available commands
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
    let cli = Cli::parse();

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

    axum::serve(listener, app).await.unwrap();
}

async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}
