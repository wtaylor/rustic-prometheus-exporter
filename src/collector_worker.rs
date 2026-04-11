use std::thread::{self, JoinHandle};

use kameo::{
    Actor,
    actor::ActorRef,
    error::Infallible,
    prelude::{Context, Message},
};
use rustic_core::CheckOptions;
use tokio::sync::mpsc::{self};
use tracing::info;

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
        // let mut file_size_total = 0;
        // for file in file_info {
        //     file_size_total += file.size;
        // }

        // let mut blob_count_total = 0;
        // let mut blob_size_total = 0;
        // let mut blob_size_uncompressed_total = 0;
        // for blob in infos_index.blobs {
        //     blob_count_total += blob.count;
        //     blob_size_total += blob.size;
        //     blob_size_uncompressed_total += blob.data_size;
        // }
        //
        // self.metric_store.set_check_success(repo_check_result);
        // self.metric_store.set_total_snapshots(snapshots.len());
        // self.metric_store.set_size_total(blob_size_total);
        // self.metric_store
        //     .set_uncompressed_size_total(blob_size_uncompressed_total);
        // self.metric_store
        //     .set_compression_ratio(blob_size_uncompressed_total as f32 / blob_size_total as f32);
        // self.metric_store.set_blob_count_total(blob_count_total);
        //
        // let scrape_duration = scrape_start.elapsed().unwrap().as_secs_f32();
        // self.metric_store
        //     .set_scrape_duration_seconds(scrape_duration);
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
        self.repo_thread_scrape_request_sender
            .as_mut()
            .unwrap()
            .send(())
            .await
            .unwrap();
    }
}

struct RepoStatisticsMessage {
    total_snapshots: usize,
}

impl Message<RepoStatisticsMessage> for CollectorWorker {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: RepoStatisticsMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.metric_store.set_total_snapshots(msg.total_snapshots);
    }
}

fn process_repo_operations(
    mut rx: mpsc::Receiver<()>,
    app_options: AppOptions,
    repo_options: RepositoryOptions,
    self_handle: ActorRef<CollectorWorker>,
) {
    let credentials = get_credentials(&app_options, &repo_options);
    info!("Opening repo");
    let repository = get_repository(&app_options, &repo_options)
        .open(&credentials)
        .unwrap();

    loop {
        info!("Waiting for message");
        rx.blocking_recv();
        info!("Received message");

        info!("Collecting snapshots");
        let snapshots = repository.get_all_snapshots().unwrap();
        info!("Collecting file info");
        let file_info = repository.infos_files().unwrap().repo;
        info!("Collecting index info");
        let infos_index = repository.infos_index().unwrap();
        info!("Running integrity checks");
        let repo_check_result = repository.check(CheckOptions::default()).unwrap();

        self_handle
            .tell(RepoStatisticsMessage {
                total_snapshots: snapshots.len(),
            })
            .blocking_send()
            .unwrap();
    }
}
