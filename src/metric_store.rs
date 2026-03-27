use metrics::{Counter, counter};

#[derive(Default)]
pub struct MetricStore {
    pub common_labels: Vec<(String, String)>,
    total_snapshots: Option<Counter>,
}

impl MetricStore {
    pub fn new(common_labels: Vec<(String, String)>) -> MetricStore {
        MetricStore {
            common_labels,
            ..Default::default()
        }
    }

    pub fn set_total_snapshots(&mut self, value: u64) {
        if self.total_snapshots.is_none() {
            let counter = counter!("restic.snapshots_total", &self.common_labels);
            counter.absolute(value);
            self.total_snapshots = Some(counter);
        } else {
            self.total_snapshots.as_ref().unwrap().absolute(value);
        }
    }
}
