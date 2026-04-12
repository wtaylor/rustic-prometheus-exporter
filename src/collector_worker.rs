use std::{
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use kameo::{
    Actor,
    actor::ActorRef,
    error::Infallible,
    prelude::{Context, Message},
};
use rustic_core::{CheckOptions, CheckResults, ConfigOptions, KeyOptions};
use tokio::sync::mpsc::{self};
use tracing::{error, info, warn};

use crate::{
    metric_store::MetricStore,
    options::{AppOptions, RepositoryOptions},
    util::{get_credentials, get_repository},
};

#[derive(Clone)]
pub struct CollectorWorkerArgs {
    pub app_options: AppOptions,
    pub repository_options: RepositoryOptions,
}

pub struct CollectorWorker {
    app_options: AppOptions,
    repo_options: RepositoryOptions,
    metric_store: MetricStore,

    repo_thread: Option<JoinHandle<()>>,
    repo_thread_scrape_request_sender: Option<mpsc::Sender<()>>,
}

impl Actor for CollectorWorker {
    type Args = CollectorWorkerArgs;
    type Error = Infallible;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        info!("Starting worker");

        let common_labels = vec![(
            "repository_name".to_string(),
            args.repository_options.name.clone(),
        )];

        info!("Started worker");

        Ok(Self {
            app_options: args.app_options,
            repo_options: args.repository_options,
            metric_store: MetricStore::new(common_labels),
            repo_thread: None,
            repo_thread_scrape_request_sender: None,
        })
    }
}

pub struct CollectMetrics;

impl Message<CollectMetrics> for CollectorWorker {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: CollectMetrics,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.repo_thread_scrape_request_sender
            .as_mut()
            .unwrap()
            .send(())
            .await
            .unwrap();
    }
}

pub struct InitialiseMessage {
    pub self_handle: ActorRef<CollectorWorker>,
}

impl Message<InitialiseMessage> for CollectorWorker {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: InitialiseMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let (repo_thread_scrape_request_sender, repo_thread_scrape_request_receiver) =
            mpsc::channel::<()>(1);

        let app_options = self.app_options.clone();
        let repo_options = self.repo_options.clone();

        self.repo_thread_scrape_request_sender = Some(repo_thread_scrape_request_sender);
        let repo_thread = thread::spawn(|| {
            process_repo_operations(
                repo_thread_scrape_request_receiver,
                app_options,
                repo_options,
                msg.self_handle,
            )
        });

        self.repo_thread = Some(repo_thread);
    }
}

struct RepoStatisticsMessage {
    scrape_duration: Duration,
    total_snapshots: usize,
    file_size_total: u64,
    repo_check_result: CheckResults,
    blob_count_total: u64,
    blob_size_total: u64,
    blob_size_uncompressed_total: u64,
    compression_ratio: f32,
}

impl Message<RepoStatisticsMessage> for CollectorWorker {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: RepoStatisticsMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.metric_store.set_check_success(msg.repo_check_result);
        self.metric_store.set_total_snapshots(msg.total_snapshots);
        self.metric_store.set_size_total(msg.blob_size_total);
        self.metric_store
            .set_uncompressed_size_total(msg.blob_size_uncompressed_total);
        self.metric_store
            .set_compression_ratio(msg.compression_ratio);
        self.metric_store.set_blob_count_total(msg.blob_count_total);
        self.metric_store
            .set_scrape_duration_seconds(msg.scrape_duration.as_secs_f32());
    }
}

fn process_repo_operations(
    mut rx: mpsc::Receiver<()>,
    app_options: AppOptions,
    repo_options: RepositoryOptions,
    self_handle: ActorRef<CollectorWorker>,
) {
    let credentials = get_credentials(&app_options, &repo_options);
    let repository = get_repository(&app_options, &repo_options);
    if repo_options.initialise {
        warn!(
            "Repository configured to initialise, this should only be done against repos you don't care about"
        );
        info!("Initialising repository");
        let init_result = repository.init(
            &credentials,
            &KeyOptions::default(),
            &ConfigOptions::default(),
        );

        match init_result {
            Ok(_) => info!("Repository successfully initialised"),
            Err(_) => error!("Failed to initialise the repository, is it already initialised?"),
        }
    }

    let repository = get_repository(&app_options, &repo_options)
        .open(&credentials)
        .unwrap();

    loop {
        rx.blocking_recv();
        let scrape_start = SystemTime::now();
        info!("Collecting snapshots");
        let snapshots = repository.get_all_snapshots().unwrap();
        info!("Collecting file info");
        let file_info = repository.infos_files().unwrap().repo;
        info!("Collecting index info");
        let infos_index = repository.infos_index().unwrap();
        info!("Running integrity checks");
        let repo_check_result = repository.check(CheckOptions::default()).unwrap();

        let mut file_size_total = 0;
        for file in file_info {
            file_size_total += file.size;
        }

        let mut blob_count_total = 0;
        let mut blob_size_total = 0;
        let mut blob_size_uncompressed_total = 0;
        for blob in infos_index.blobs {
            blob_count_total += blob.count;
            blob_size_total += blob.size;
            blob_size_uncompressed_total += blob.data_size;
        }

        let total_snapshots = snapshots.len();
        let compression_ratio = blob_size_uncompressed_total as f32 / blob_size_total as f32;
        let scrape_duration = scrape_start.elapsed().unwrap();

        self_handle
            .tell(RepoStatisticsMessage {
                scrape_duration,
                file_size_total,
                repo_check_result,
                blob_count_total,
                blob_size_total,
                blob_size_uncompressed_total,
                compression_ratio,
                total_snapshots,
            })
            .blocking_send()
            .unwrap();
    }
}
