use std::path::PathBuf;

use metrics::counter;
use rustic_backend::BackendOptions;
use rustic_core::{Credentials, Repository};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::options::{AppOptions, RepositoryOptions};

pub async fn start(settings: AppOptions, cancellation_token: CancellationToken) {
    info!("Starting collector scheduler");

    repository_worker(settings.restic.rpe_cache, &settings.restic.repositories[0]).await;
}

async fn repository_worker(cache_dir: Option<PathBuf>, repository_options: &RepositoryOptions) {
    let backend = BackendOptions::default()
        .repository(&repository_options.url)
        .to_backends()
        .unwrap();

    let mut repo_options = rustic_core::RepositoryOptions::default();
    match cache_dir {
        Some(value) => {
            repo_options = repo_options.cache_dir(value);
        }
        None => {
            repo_options = repo_options.no_cache(true);
        }
    }

    let repo_credentials = Credentials::password(&repository_options.password);

    let restic_repository = Repository::new(&repo_options, &backend).unwrap();

    info!("Opening repository");
    match restic_repository.open(&repo_credentials) {
        Ok(opened_repository) => {
            info!("Getting all snapshots");
            let snapshots = opened_repository.get_all_snapshots().unwrap();

            info!("Found {} snapshots", snapshots.len());

            counter!("restic.snapshot_count").absolute(snapshots.len() as u64);
            counter!("restic.snapshot_count", "second_repo" => "test")
                .absolute(snapshots.len() as u64);
        }
        Err(e) => {
            eprintln!("Error opening repository: {}", e);
        }
    }
}
