use std::time::Duration;

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

pub struct PostRepositoryMetricsMessage {
    pub repository_ref: RepositoryRef,
    pub repository_metrics: RepositoryMetricsSnapshot,
    pub scrape_metrics: ScrapeMetricsSnapshot,
}

impl Message<PostRepositoryMetricsMessage> for MetricsExporter {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: PostRepositoryMetricsMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let repository_metric_store = match self.repository_metrics.get(&msg.repository_ref) {
            Some(store) => store,
            None => {
                let common_labels =
                    vec![("repository_name".to_string(), msg.repository_ref.0.clone())];

                self.repository_metrics.insert(
                    msg.repository_ref.clone(),
                    RepositoryMetricStore::new(&common_labels),
                );
                self.repository_metrics
                    .get(&msg.repository_ref)
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
    }
}
