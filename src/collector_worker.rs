use std::time::SystemTime;

use kameo::{
    Actor,
    error::Infallible,
    prelude::{Context, Message},
};
use rustic_core::{CheckOptions, OpenStatus, Repository};
use tracing::info;

use crate::{
    metric_store::MetricStore,
    options::{AppOptions, RepositoryOptions},
    util::{get_credentials, get_repository},
};

pub struct CollectorWorkerArgs {
    pub app_options: AppOptions,
    pub repository_options: RepositoryOptions,
}

pub struct CollectorWorker {
    pub repository: Repository<OpenStatus>,
    metric_store: MetricStore,
}

impl Actor for CollectorWorker {
    type Args = CollectorWorkerArgs;
    type Error = Infallible;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let repository = get_repository(&args.app_options, &args.repository_options);
        let credentials = get_credentials(&args.app_options, &args.repository_options);
        let repository = repository.open(&credentials).unwrap();
        let common_labels = vec![(
            "repository_name".to_string(),
            args.repository_options.name.clone(),
        )];

        Ok(Self {
            repository,
            metric_store: MetricStore::new(common_labels),
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
        let scrape_start = SystemTime::now();

        info!("Starting scrape");
        info!("Collecting snapshots");
        let snapshots = self.repository.get_all_snapshots().unwrap();
        info!("Collecting file info");
        let file_info = self.repository.infos_files().unwrap().repo;
        info!("Collecting index info");
        let infos_index = self.repository.infos_index().unwrap();
        info!("Running integrity checks");
        let repo_check_result = self.repository.check(CheckOptions::default()).unwrap();

        // let mut file_size_total = 0;
        // for file in file_info {
        //     file_size_total += file.size;
        // }

        let mut blob_count_total = 0;
        let mut blob_size_total = 0;
        let mut blob_size_uncompressed_total = 0;
        for blob in infos_index.blobs {
            blob_count_total += blob.count;
            blob_size_total += blob.size;
            blob_size_uncompressed_total += blob.data_size;
        }

        self.metric_store.set_check_success(repo_check_result);
        self.metric_store.set_total_snapshots(snapshots.len());
        self.metric_store.set_size_total(blob_size_total);
        self.metric_store
            .set_uncompressed_size_total(blob_size_uncompressed_total);
        self.metric_store
            .set_compression_ratio(blob_size_uncompressed_total as f32 / blob_size_total as f32);
        self.metric_store.set_blob_count_total(blob_count_total);

        let scrape_duration = scrape_start.elapsed().unwrap().as_secs_f32();
        self.metric_store
            .set_scrape_duration_seconds(scrape_duration);
    }
}
