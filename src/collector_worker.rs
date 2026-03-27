use kameo::{
    Actor,
    error::Infallible,
    prelude::{Context, Message},
};
use metrics::{Counter, counter};
use rustic_core::{Credentials, OpenStatus, Repository};
use tracing::info;

use crate::options::{AppOptions, RepositoryOptions};

pub struct CollectorWorkerArgs {
    pub app_options: AppOptions,
    pub repository_options: RepositoryOptions,
}

pub struct CollectorWorker {
    pub repository: Repository<OpenStatus>,
    repository_metric_handles: RepositoryMetricHandles,
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
            repository_metric_handles: RepositoryMetricHandles {
                common_labels,
                ..Default::default()
            },
        })
    }
}

fn get_credentials(
    app_options: &AppOptions,
    repository_options: &RepositoryOptions,
) -> Credentials {
    let password = app_options
        .restic
        .defaults
        .as_ref()
        .and_then(|d| d.password.as_ref())
        .or(repository_options.password.as_ref())
        .unwrap();
    Credentials::password(password)
}

fn get_repository(
    app_options: &AppOptions,
    repository_options: &RepositoryOptions,
) -> Repository<()> {
    let backend_protocol = repository_options.url.split(':').next().unwrap_or("local");

    let mut backend_options = app_options
        .restic
        .defaults
        .as_ref()
        .and_then(|d| d.backend_options.as_ref())
        .and_then(|d| d.get(backend_protocol))
        .and_then(|d| Some(d.clone()))
        .unwrap_or_default();

    if let Some(repo_backend_options) = repository_options.backend_options.clone() {
        backend_options.extend(repo_backend_options);
    }

    let backend = rustic_backend::BackendOptions::default()
        .repository(&repository_options.url)
        .options(backend_options)
        .to_backends()
        .unwrap();

    let mut repo_options = rustic_core::RepositoryOptions::default();
    match app_options.restic.cache_dir.as_deref() {
        Some(value) => {
            repo_options = repo_options.cache_dir(value);
        }
        None => {
            repo_options = repo_options.no_cache(true);
        }
    }

    Repository::new(&repo_options, &backend).unwrap()
}

#[derive(Default)]
struct RepositoryMetricHandles {
    common_labels: Vec<(String, String)>,
    total_snapshots: Option<Counter>,
}

impl RepositoryMetricHandles {
    fn set_total_snapshots(&mut self, value: u64) {
        if self.total_snapshots.is_none() {
            let counter = counter!("restic.snapshots_total", &self.common_labels);
            counter.absolute(value);
            self.total_snapshots = Some(counter);
        } else {
            self.total_snapshots.as_ref().unwrap().absolute(value);
        }
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
        info!("Getting all snapshots");
        let snapshots = self.repository.get_all_snapshots().unwrap();

        info!("Found {} snapshots", snapshots.len());

        self.repository_metric_handles
            .set_total_snapshots(snapshots.len() as u64);
    }
}
