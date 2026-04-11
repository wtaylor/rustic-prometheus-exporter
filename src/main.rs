use std::path::PathBuf;

use axum::{Router, extract::State, routing::get};
use clap::{Parser, Subcommand};
use config::{Config, File};
use kameo::actor::{ActorRef, Spawn};
use kameo_actors::scheduler::{Scheduler, SetInterval};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{
    collector_supervisor::{CollectorSchedulerArgs, CollectorSupervisor, RequestCollectionMessage},
    options::AppOptions,
};

mod collector_supervisor;
mod collector_worker;
mod metric_store;
mod options;
mod util;

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

    let app_options = settings_monitor.try_deserialize::<AppOptions>().unwrap();

    let metrics_handle = PrometheusBuilder::new().install_recorder().unwrap();
    let state = AppState { metrics_handle };

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(app_options.http.listen)
        .await
        .unwrap();

    info!("listening on {}", app_options.http.listen);
    info!(
        "metrics endpoint is available at http://{}/metrics",
        app_options.http.listen
    );

    let scheduler = Scheduler::spawn(Scheduler::new());

    let collector_supervisor = CollectorSupervisor::spawn(CollectorSchedulerArgs {
        app_options: app_options.clone(),
    });

    let collection_interval = SetInterval::new(
        collector_supervisor.downgrade(),
        app_options.collector.interval,
        RequestCollectionMessage,
    );
    scheduler.tell(collection_interval).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(
            shutdown_token.clone(),
            collector_supervisor,
            scheduler,
        ))
        .await
        .unwrap();
}

async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}

async fn shutdown_signal(
    cancellation_token: CancellationToken,
    supervisor_ref: ActorRef<CollectorSupervisor>,
    sheduler_ref: ActorRef<Scheduler>,
) {
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
    sheduler_ref.kill();
    supervisor_ref.kill();
}
