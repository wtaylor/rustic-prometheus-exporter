use std::{
    collections::{BTreeMap, HashMap},
    net::SocketAddr,
    path::PathBuf,
    time::Duration,
};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppOptions {
    pub http: HttpOptions,
    pub restic: ResticOptions,
    pub collector: CollectorOptions,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpOptions {
    pub listen: SocketAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResticOptions {
    pub cache_dir: Option<PathBuf>,
    pub defaults: Option<ResticDefaultOptions>,
    pub repositories: Vec<RepositoryOptions>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResticDefaultOptions {
    pub password: Option<String>,
    pub backend_options: Option<HashMap<String, BTreeMap<String, String>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RepositoryOptions {
    pub name: String,
    pub url: String,
    pub password: Option<String>,
    pub backend_options: Option<BTreeMap<String, String>>,
    pub additional_labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectorOptions {
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    pub metrics: Option<Vec<MetricOption>>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum MetricOption {
    CheckSuccess,
    LocksTotal,
    ScrapeDurationSeconds,
    SizeTotal,
    UncompressedSizeTotal,
    CompressionRation,
    BlobCountTotal,
    SnapshotsTotal,
    BackupTimestamp,
    BackupSnapshotsTotal,
    BackupFilesTotal,
    BackupSizeTotal,
    BackupFilesNew,
    BackupFilesChanged,
    BackupFilesUnmodified,
    BackupDirsNew,
    BackupDirsChanged,
    BackupDirsUnmodified,
    BackupDataAddedBytes,
    BackupDurationSeconds,
}
