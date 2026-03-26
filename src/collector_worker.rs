use kameo::{Actor, error::Infallible};
use rustic_core::{Credentials, OpenStatus, Repository};

use crate::options::{AppOptions, RepositoryOptions};

pub struct CollectorWorkerArgs {
    pub app_options: AppOptions,
    pub repository_options: RepositoryOptions,
}

pub struct CollectorWorker {
    pub app_options: AppOptions,
    pub repository_options: RepositoryOptions,
    pub repository: Repository<OpenStatus>,
}

impl Actor for CollectorWorker {
    type Args = CollectorWorkerArgs;
    type Error = Infallible;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let repository = get_repository(&args.app_options, &args.repository_options);
        let credentials = get_credentials(&args.app_options, &args.repository_options);
        let repository = repository.open(&credentials).unwrap();

        Ok(Self {
            app_options: args.app_options,
            repository_options: args.repository_options,
            repository,
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
