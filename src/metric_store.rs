use metrics::{Counter, Gauge, counter, gauge};
use rustic_core::CheckResults;

#[derive(Default)]
pub struct MetricStore {
    pub common_labels: Vec<(String, String)>,
    total_snapshots: Option<Gauge>,
    check_success: Option<Gauge>,
    locks_total: Option<Gauge>,
    scrape_duration_seconds: Option<Gauge>,
    size_total: Option<Gauge>,
    uncompressed_size_total: Option<Gauge>,
    compression_ratio: Option<Gauge>,
    blob_count_total: Option<Gauge>,
}

impl MetricStore {
    pub fn new(common_labels: Vec<(String, String)>) -> MetricStore {
        MetricStore {
            common_labels,
            ..Default::default()
        }
    }

    pub fn set_total_snapshots(&mut self, value: usize) {
        if self.total_snapshots.is_none() {
            let guage = gauge!("restic.snapshots_total", &self.common_labels);
            guage.set(value as f32);
            self.total_snapshots = Some(guage);
        } else {
            self.total_snapshots.as_ref().unwrap().set(value as f32);
        }
    }

    pub fn set_check_success(&mut self, value: CheckResults) {
        let value = if value.is_ok().is_ok() { 1 } else { 0 };
        if self.check_success.is_none() {
            let guage = gauge!("restic.check_success", &self.common_labels);
            guage.set(value);
            self.check_success = Some(guage);
        } else {
            self.check_success.as_ref().unwrap().set(value);
        }
    }

    pub fn set_locks_total(&mut self, value: u8) {
        if self.locks_total.is_none() {
            let guage = gauge!("restic.locks_total", &self.common_labels);
            guage.set(value);
            self.locks_total = Some(guage);
        } else {
            self.locks_total.as_ref().unwrap().set(value);
        }
    }

    pub fn set_scrape_duration_seconds(&mut self, value: f32) {
        if self.scrape_duration_seconds.is_none() {
            let guage = gauge!("restic.scrape_duration_seconds", &self.common_labels);
            guage.set(value);
            self.scrape_duration_seconds = Some(guage);
        } else {
            self.scrape_duration_seconds.as_ref().unwrap().set(value);
        }
    }

    pub fn set_size_total(&mut self, value: u64) {
        if self.size_total.is_none() {
            let guage = gauge!("restic.size_total", &self.common_labels);
            guage.set(value as f64);
            self.size_total = Some(guage);
        } else {
            self.size_total.as_ref().unwrap().set(value as f64);
        }
    }

    pub fn set_uncompressed_size_total(&mut self, value: u64) {
        if self.uncompressed_size_total.is_none() {
            let guage = gauge!("restic.uncompressed_size_total", &self.common_labels);
            guage.set(value as f64);
            self.uncompressed_size_total = Some(guage);
        } else {
            self.uncompressed_size_total
                .as_ref()
                .unwrap()
                .set(value as f64);
        }
    }

    pub fn set_blob_count_total(&mut self, value: u64) {
        if self.blob_count_total.is_none() {
            let guage = gauge!("restic.blob_count_total", &self.common_labels);
            guage.set(value as f64);
            self.blob_count_total = Some(guage);
        } else {
            self.blob_count_total.as_ref().unwrap().set(value as f64);
        }
    }

    pub fn set_compression_ratio(&mut self, value: f32) {
        if self.compression_ratio.is_none() {
            let guage = gauge!("restic.compression_ratio", &self.common_labels);
            guage.set(value as f64);
            self.compression_ratio = Some(guage);
        } else {
            self.compression_ratio.as_ref().unwrap().set(value);
        }
    }
}
