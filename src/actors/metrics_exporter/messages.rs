use std::{f64, time::Duration};

use kameo::prelude::{Context, Message};

use crate::actors::metrics_exporter::{MetricsExporter, RepositoryMetricStore};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct RepositoryRef(pub String);

pub struct RepositoryMetricsSnapshot {
    pub total_snapshots: usize,
    pub check_result: bool,
    pub blob_count_total: u64,
    pub blob_size_total: u64,
    pub blob_size_uncompressed_total: u64,
}

pub struct ScrapeMetricsSnapshot {
    pub scrape_duration: Duration,
}

pub struct SnapshotMetricsSnapshot {
    pub started_ms: i64,
    pub finished_ms: i64,
    pub duration_ms: f64,
    pub size_total_bytes: u64,
    pub size_added_bytes: u64,
    pub files_new: u64,
    pub files_changed: u64,
    pub files_unmodified: u64,
    pub files_total: u64,
    pub dirs_new: u64,
    pub dirs_changed: u64,
    pub dirs_unmodified: u64,
    pub dirs_total: u64,
}

pub struct PostRepositoryMetricsMessage {
    pub repository_ref: RepositoryRef,
    pub repository_metrics: RepositoryMetricsSnapshot,
    pub last_snapshot_metrics: Option<SnapshotMetricsSnapshot>,
    pub scrape_metrics: ScrapeMetricsSnapshot,
}

impl Message<PostRepositoryMetricsMessage> for MetricsExporter {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: PostRepositoryMetricsMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let repository_metric_store = match self.repository_metrics.get_mut(&msg.repository_ref) {
            Some(store) => store,
            None => {
                let common_labels =
                    vec![("repository_name".to_string(), msg.repository_ref.0.clone())];

                self.repository_metrics.insert(
                    msg.repository_ref.clone(),
                    RepositoryMetricStore::new(&common_labels),
                );
                self.repository_metrics
                    .get_mut(&msg.repository_ref)
                    .expect("fatal error writing to hashmap and then reading same key")
            }
        };

        repository_metric_store
            .total_snapshots
            .set(msg.repository_metrics.total_snapshots as u32);

        repository_metric_store
            .check_success
            .set(msg.repository_metrics.check_result as i8);

        repository_metric_store
            .scrape_duration_seconds
            .set(msg.scrape_metrics.scrape_duration.as_secs_f32());

        repository_metric_store
            .blob_count_total
            .set(msg.repository_metrics.blob_count_total as f64);

        let blob_size_total = msg.repository_metrics.blob_size_total as f64;
        let blob_size_uncompressed_total =
            msg.repository_metrics.blob_size_uncompressed_total as f64;

        let compression_ratio = blob_size_uncompressed_total / blob_size_total;

        repository_metric_store.size_total.set(blob_size_total);

        repository_metric_store
            .uncompressed_size_total
            .set(blob_size_uncompressed_total);

        repository_metric_store
            .compression_ratio
            .set(compression_ratio);

        if let Some(snapshot_metrics) = msg.last_snapshot_metrics {
            let snapshot_metric_store = repository_metric_store.get_or_init_snapshot_store();

            snapshot_metric_store
                .started
                .set(snapshot_metrics.started_ms as f64);

            snapshot_metric_store
                .finished
                .set(snapshot_metrics.finished_ms as f64);

            snapshot_metric_store
                .duration
                .set(snapshot_metrics.duration_ms);

            snapshot_metric_store
                .size_total_bytes
                .set(snapshot_metrics.size_total_bytes as f64);

            snapshot_metric_store
                .size_added_bytes
                .set(snapshot_metrics.size_added_bytes as f64);

            snapshot_metric_store
                .files_new
                .set(snapshot_metrics.files_new as f64);

            snapshot_metric_store
                .files_changed
                .set(snapshot_metrics.files_changed as f64);

            snapshot_metric_store
                .files_unmodified
                .set(snapshot_metrics.files_unmodified as f64);

            snapshot_metric_store
                .files_total
                .set(snapshot_metrics.files_total as f64);

            snapshot_metric_store
                .dirs_new
                .set(snapshot_metrics.dirs_new as f64);
            snapshot_metric_store
                .dirs_changed
                .set(snapshot_metrics.dirs_changed as f64);
            snapshot_metric_store
                .dirs_unmodified
                .set(snapshot_metrics.dirs_unmodified as f64);
            snapshot_metric_store
                .dirs_total
                .set(snapshot_metrics.dirs_total as f64);
        }
    }
}
