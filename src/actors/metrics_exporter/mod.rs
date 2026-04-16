use std::collections::HashMap;

use kameo::{Actor, error::Infallible};
use metrics::{Gauge, gauge};

use crate::actors::metrics_exporter::messages::RepositoryRef;

pub mod messages;

pub struct SnapshotMetricStore {
    pub started: Gauge,
    pub finished: Gauge,
    pub duration: Gauge,
    pub size_total_bytes: Gauge,
    pub size_added_bytes: Gauge,
    pub files_new: Gauge,
    pub files_changed: Gauge,
    pub files_unmodified: Gauge,
    pub files_total: Gauge,
    pub dirs_new: Gauge,
    pub dirs_changed: Gauge,
    pub dirs_unmodified: Gauge,
    pub dirs_total: Gauge,
}

pub struct RepositoryMetricStore {
    pub common_labels: Vec<(String, String)>,
    pub total_snapshots: Gauge,
    pub check_success: Gauge,
    pub scrape_duration_seconds: Gauge,
    pub size_total: Gauge,
    pub uncompressed_size_total: Gauge,
    pub compression_ratio: Gauge,
    pub blob_count_total: Gauge,

    pub last_snapshot_store: Option<SnapshotMetricStore>,
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
            last_snapshot_store: None,
            common_labels: common_labels.clone(),
        }
    }

    pub fn get_or_init_snapshot_store(&mut self) -> &SnapshotMetricStore {
        if self.last_snapshot_store.is_some() {
            return self.last_snapshot_store.as_ref().unwrap();
        }

        self.last_snapshot_store = Some(SnapshotMetricStore {
            started: gauge!("restic.snapshot.started", &self.common_labels),
            finished: gauge!("restic.snapshot.finished", &self.common_labels),
            duration: gauge!("restic.snapshot.duration", &self.common_labels),
            size_added_bytes: gauge!("restic.snapshot.size_added_bytes", &self.common_labels),
            size_total_bytes: gauge!("restic.snapshot.size_total_bytes", &self.common_labels),
            files_new: gauge!("restic.snapshot.files_new", &self.common_labels),
            files_changed: gauge!("restic.snapshot.files_changed", &self.common_labels),
            files_unmodified: gauge!("restic.snapshot.files_unmodified", &self.common_labels),
            files_total: gauge!("restic.snapshot.files_total", &self.common_labels),
            dirs_new: gauge!("restic.snapshot.dirs_new", &self.common_labels),
            dirs_changed: gauge!("restic.snapshot.dirs_changed", &self.common_labels),
            dirs_unmodified: gauge!("restic.snapshot.dirs_unmodified", &self.common_labels),
            dirs_total: gauge!("restic.snapshot.dirs_total", &self.common_labels),
        });

        self.last_snapshot_store.as_ref().unwrap()
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
