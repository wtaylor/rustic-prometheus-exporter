use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use axum::{Router, extract::State, routing::get};
use clap::{Parser, Subcommand};
use config::{Case, Config, File};
use kameo::actor::{ActorRef, Spawn};
use kameo_actors::scheduler::{Scheduler, SetInterval};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tokio::signal;
use tracing::info;

use crate::{
    actors::{
        collector_supervisor::{
            CollectorSupervisor, CollectorSupervisorArgs, messages::RequestCollectionMessage,
        },
        metrics_exporter::MetricsExporter,
    },
    options::AppOptions,
};

mod actors;
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

fn get_config(config_path: PathBuf) -> Result<AppOptions> {
    let config = Config::builder()
        .add_source(File::from(config_path))
        .add_source(
            config::Environment::with_prefix("RPE")
                .separator("__")
                .convert_case(Case::Lower),
        )
        .build()?;

    Ok(config.try_deserialize::<AppOptions>()?)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    info!("reading config from: {:?}", cli.config);

    let config = get_config(cli.config.clone()).with_context(|| {
        format!(
            "failed to derive a valid configuration from {:?}",
            cli.config
        )
    })?;

    let metrics_handle = PrometheusBuilder::new()
        .install_recorder()
        .with_context(|| "failed to setup the prometheus recorder")?;

    let state = AppState { metrics_handle };

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.http.listen).await?;

    info!("listening on {}", config.http.listen);
    info!(
        "metrics endpoint is available at http://{}/metrics",
        config.http.listen
    );

    let metrics_exporter = MetricsExporter::spawn(());
    let collector_supervisor = CollectorSupervisor::spawn(CollectorSupervisorArgs {
        app_options: config.clone(),
        metrics_exporter_ref: metrics_exporter,
    });

    let scheduler = Scheduler::spawn(Scheduler::new());
    let collection_interval = SetInterval::new(
        collector_supervisor.downgrade(),
        config.collector.interval,
        RequestCollectionMessage,
    );
    scheduler.tell(collection_interval).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(collector_supervisor, scheduler))
        .await?;

    Ok(())
}

async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}

async fn shutdown_signal(
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
    sheduler_ref.kill();
    supervisor_ref.kill();
}
