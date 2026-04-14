use std::{
    thread::{self, JoinHandle},
    time::SystemTime,
};

use kameo::{Actor, actor::ActorRef, error::Infallible};
use rustic_core::{CheckOptions, ConfigOptions, KeyOptions};
use tokio::sync::mpsc::{self};
use tracing::{error, info, warn};

use crate::{
    actors::metrics_exporter::{
        MetricsExporter,
        messages::{
            PostRepositoryMetricsMessage, RepositoryMetricsSnapshot, RepositoryRef,
            ScrapeMetricsSnapshot,
        },
    },
    options::{AppOptions, RepositoryOptions},
    util::{get_credentials, get_repository},
};

pub mod messages;

#[derive(Clone)]
pub struct CollectorWorkerArgs {
    pub app_options: AppOptions,
    pub repository_options: RepositoryOptions,
    pub exporter_ref: ActorRef<MetricsExporter>,
}

pub struct CollectorWorker {
    _repo_thread: JoinHandle<()>,
    repo_thread_scrape_request_sender: mpsc::Sender<()>,
}

impl Actor for CollectorWorker {
    type Args = CollectorWorkerArgs;
    type Error = Infallible;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let (repo_thread_scrape_request_sender, repo_thread_scrape_request_receiver) =
            mpsc::channel::<()>(1);

        let app_options = args.app_options.clone();
        let repository_options = args.repository_options.clone();
        let repository_ref = RepositoryRef(args.repository_options.name.clone());
        let repo_thread = thread::spawn(|| {
            process_repo_operations(
                repo_thread_scrape_request_receiver,
                repository_ref,
                app_options,
                repository_options,
                args.exporter_ref,
            )
        });

        Ok(Self {
            _repo_thread: repo_thread,
            repo_thread_scrape_request_sender,
        })
    }
}

fn process_repo_operations(
    mut rx: mpsc::Receiver<()>,
    repository_ref: RepositoryRef,
    app_options: AppOptions,
    repo_options: RepositoryOptions,
    exporter_handle: ActorRef<MetricsExporter>,
) {
    let credentials = get_credentials(&app_options, &repo_options).unwrap();
    let repository = get_repository(&app_options, &repo_options).unwrap();
    if repo_options.initialise.is_some_and(|i| i) {
        warn!(
            "Repository configured to initialise, this should only be done against repos you don't care about"
        );
        info!("Initialising repository");
        let init_result = repository.clone().init(
            &credentials,
            &KeyOptions::default(),
            &ConfigOptions::default(),
        );

        match init_result {
            Ok(_) => info!("Repository successfully initialised"),
            Err(_) => error!("Failed to initialise the repository, is it already initialised?"),
        }
    }

    let repository = repository.open(&credentials).unwrap();

    loop {
        rx.blocking_recv();
        let scrape_start = SystemTime::now();
        info!("Collecting snapshots");
        let snapshots = repository.get_all_snapshots().unwrap();
        info!("Collecting file info");
        let _file_info = repository.infos_files().unwrap().repo;
        info!("Collecting index info");
        let infos_index = repository.infos_index().unwrap();
        info!("Running integrity checks");
        let repo_check_result = repository.check(CheckOptions::default()).unwrap();

        let mut blob_count_total = 0;
        let mut blob_size_total = 0;
        let mut blob_size_uncompressed_total = 0;
        for blob in infos_index.blobs {
            blob_count_total += blob.count;
            blob_size_total += blob.size;
            blob_size_uncompressed_total += blob.data_size;
        }

        let total_snapshots = snapshots.len();
        let scrape_duration = scrape_start.elapsed().unwrap();

        let repository_metrics = RepositoryMetricsSnapshot {
            check_result: repo_check_result.is_ok().is_ok(),
            blob_count_total,
            blob_size_total,
            blob_size_uncompressed_total,
            total_snapshots,
        };

        let scrape_metrics = ScrapeMetricsSnapshot { scrape_duration };

        exporter_handle
            .tell(PostRepositoryMetricsMessage {
                repository_ref: repository_ref.clone(),
                repository_metrics,
                scrape_metrics,
            })
            .blocking_send()
            .unwrap();
    }
}
