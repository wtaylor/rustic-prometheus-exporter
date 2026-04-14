use kameo::prelude::Message;

use crate::actors::collector_supervisor::CollectorSupervisor;
use crate::actors::collector_worker::messages::CollectMetricsMessage;

#[derive(Clone)]
pub struct RequestCollectionMessage;

impl Message<RequestCollectionMessage> for CollectorSupervisor {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: RequestCollectionMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        for worker in self.workers.iter() {
            let worker = worker.1.clone();
            worker.tell(CollectMetricsMessage {}).await.unwrap();
        }
    }
}
