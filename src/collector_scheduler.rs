use std::collections::HashMap;

use kameo::{
    Actor,
    actor::{ActorRef, Spawn},
    error::Infallible,
};
use tokio::{task::JoinHandle, time};
use tracing::info;

use crate::{
    collector_worker::{CollectMetrics, CollectorWorker, CollectorWorkerArgs},
    options::AppOptions,
};

pub struct CollectorSchedulerArgs {
    pub app_options: AppOptions,
}

pub struct CollectorScheduler {
    workers: HashMap<String, ActorRef<CollectorWorker>>,
    _collect_schedule: JoinHandle<()>,
}

impl Actor for CollectorScheduler {
    type Args = CollectorSchedulerArgs;
    type Error = Infallible;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
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

        let active_workers = workers.clone();
        let collect_schedule = tokio::spawn(async move {
            let mut interval = time::interval(args.app_options.collector.interval);
            loop {
                interval.tick().await;
                for worker in active_workers.clone() {
                    worker.1.tell(CollectMetrics {}).await.unwrap();
                }
            }
        });

        Ok(Self {
            workers,
            _collect_schedule: collect_schedule,
        })
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
