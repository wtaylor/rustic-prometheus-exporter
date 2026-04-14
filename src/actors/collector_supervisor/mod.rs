use std::{collections::HashMap, time::Duration};

pub mod messages;

use kameo::{
    Actor,
    actor::{ActorRef, Spawn},
    error::Infallible,
    supervision::{RestartPolicy, SupervisionStrategy},
};
use tokio::task::JoinSet;
use tracing::info;

use crate::{
    actors::{
        collector_worker::{CollectorWorker, CollectorWorkerArgs},
        metrics_exporter::MetricsExporter,
    },
    options::AppOptions,
};

pub struct CollectorSupervisorArgs {
    pub app_options: AppOptions,
    pub metrics_exporter_ref: ActorRef<MetricsExporter>,
}

pub struct CollectorSupervisor {
    workers: HashMap<String, ActorRef<CollectorWorker>>,
}

impl Actor for CollectorSupervisor {
    type Args = CollectorSupervisorArgs;
    type Error = Infallible;

    fn supervision_strategy() -> kameo::supervision::SupervisionStrategy {
        SupervisionStrategy::OneForOne
    }

    async fn on_start(
        args: Self::Args,
        supervisor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        info!("Starting collector supervisor");

        let worker_options = args.app_options.restic.repositories.iter();
        let mut worker_spawn_set = JoinSet::new();
        for options in worker_options {
            let supervisor_ref = supervisor_ref.clone();
            let app_options = args.app_options.clone();
            let exporter_ref = args.metrics_exporter_ref.clone();
            let options = options.clone();
            worker_spawn_set.spawn(async move {
                (
                    options.name.clone(),
                    CollectorWorker::supervise(
                        &supervisor_ref,
                        CollectorWorkerArgs {
                            exporter_ref: exporter_ref,
                            app_options: app_options,
                            repository_options: options,
                        },
                    )
                    .restart_policy(RestartPolicy::Permanent)
                    .restart_limit(1, Duration::from_secs(2))
                    .spawn()
                    .await,
                )
            });
        }

        let workers: HashMap<String, ActorRef<CollectorWorker>> =
            worker_spawn_set.join_all().await.into_iter().collect();

        Ok(Self { workers })
    }

    async fn on_stop(
        &mut self,
        _actor_ref: kameo::prelude::WeakActorRef<Self>,
        _reason: kameo::prelude::ActorStopReason,
    ) -> Result<(), Self::Error> {
        for worker in self.workers.iter() {
            worker.1.kill();
        }

        Ok(())
    }
}
