use std::collections::HashMap;

use kameo::{
    Actor,
    actor::{ActorRef, Spawn},
    error::Infallible,
};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{
    collector_worker::{CollectorWorker, CollectorWorkerArgs},
    options::AppOptions,
};

pub struct CollectorSchedulerArgs {
    pub app_options: AppOptions,
}

pub struct CollectorScheduler {
    app_options: AppOptions,
    workers: HashMap<String, ActorRef<CollectorWorker>>,
}

impl Actor for CollectorScheduler {
    type Args = CollectorSchedulerArgs;
    type Error = Infallible;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        info!("Starting collector scheduler");
        let workers = args
            .app_options
            .restic
            .repositories
            .iter()
            .map(|r| {
                (
                    r.name.clone(),
                    CollectorWorker::spawn(CollectorWorkerArgs {
                        app_options: args.app_options.clone(),
                        repository_options: r.clone(),
                    }),
                )
            })
            .collect::<HashMap<String, ActorRef<CollectorWorker>>>();
        Ok(Self {
            app_options: args.app_options,
            workers,
        })
    }

    async fn on_stop(
        &mut self,
        actor_ref: kameo::prelude::WeakActorRef<Self>,
        reason: kameo::prelude::ActorStopReason,
    ) -> Result<(), Self::Error> {
        for worker in self.workers.iter() {
            worker.1.kill();
        }

        Ok(())
    }
}

// async fn repository_worker(settings: &AppOptions, repository_options: &RepositoryOptions) {
//     let backend_options = get_repo_backend_options(
//         &settings.restic.defaults,
//         &repository_options,
//     );
//
//     let backend = rustic_backend::BackendOptions::default()
//         .repository(&repository_options.url)
//         .options(backend_options)
//         .to_backends()
//         .unwrap();
//
//     let mut repo_options = rustic_core::RepositoryOptions::default();
//     match settings.restic.cache_dir.as_deref() {
//         Some(value) => {
//             repo_options = repo_options.cache_dir(value);
//         }
//         None => {
//             repo_options = repo_options.no_cache(true);
//         }
//     }
//
//     let repo_credentials = Credentials::password(&repository_options.password);
//
//     let restic_repository = Repository::new(&repo_options, &backend).unwrap();
//
//     info!("Opening repository");
//     match restic_repository.open(&repo_credentials) {
//         Ok(opened_repository) => {
//             info!("Getting all snapshots");
//             let snapshots = opened_repository.get_all_snapshots().unwrap();
//
//             info!("Found {} snapshots", snapshots.len());
//
//             counter!("restic.snapshot_count", "repo" => format!("{}", &repository_options.name))
//                 .absolute(snapshots.len() as u64);
//         }
//         Err(e) => {
//             eprintln!("Error opening repository: {}", e);
//         }
//     }
// }
