use std::collections::HashMap;

use kameo::{Actor, error::Infallible};
use metrics::{Gauge, gauge};

use crate::actors::metrics_exporter::messages::RepositoryRef;

pub mod messages;

pub struct RepositoryMetricStore {
    pub _common_labels: Vec<(String, String)>,
    pub total_snapshots: Gauge,
    pub check_success: Gauge,
    pub scrape_duration_seconds: Gauge,
    pub size_total: Gauge,
    pub uncompressed_size_total: Gauge,
    pub compression_ratio: Gauge,
    pub blob_count_total: Gauge,
}

impl RepositoryMetricStore {
    pub fn new(common_labels: &Vec<(String, String)>) -> RepositoryMetricStore {
        Self {
            total_snapshots: gauge!("restic.snapshots_total", common_labels),
            check_success: gauge!("restic.check_success", common_labels),
            scrape_duration_seconds: gauge!("restic.scrape_duration_seconds", common_labels),
            size_total: gauge!("restic.size_total", common_labels),
            uncompressed_size_total: gauge!("restic.uncompressed_size_total", common_labels),
            compression_ratio: gauge!("restic.compression_ratio", common_labels),
            blob_count_total: gauge!("restic.blob_count_total", common_labels),
            _common_labels: common_labels.clone(),
        }
    }
}

pub struct MetricsExporter {
    repository_metrics: HashMap<RepositoryRef, RepositoryMetricStore>,
}

impl Actor for MetricsExporter {
    type Args = ();
    type Error = Infallible;

    async fn on_start(
        _args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            repository_metrics: HashMap::new(),
        })
    }
}
