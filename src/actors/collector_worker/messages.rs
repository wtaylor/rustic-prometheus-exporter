use kameo::prelude::{Context, Message};

use crate::actors::collector_worker::CollectorWorker;

pub struct CollectMetricsMessage;

impl Message<CollectMetricsMessage> for CollectorWorker {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: CollectMetricsMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.repo_thread_scrape_request_sender
            .send(())
            .await
            .unwrap();
    }
}
