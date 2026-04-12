use std::{
    collections::{BTreeMap, HashMap},
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
    time::Duration,
};

use rustic_backend::SupportedBackend;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppOptions {
    pub http: HttpOptions,
    pub restic: ResticOptions,
    pub collector: CollectorOptions,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpOptions {
    pub listen: SocketAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResticOptions {
    pub cache_dir: Option<PathBuf>,
    pub defaults: Option<ResticDefaultOptions>,
    pub repositories: Vec<RepositoryOptions>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResticDefaultOptions {
    pub password: Option<String>,
    pub backend_options: Option<HashMap<String, BTreeMap<String, String>>>,
}

impl ResticDefaultOptions {
    pub fn get_options_for_backend(
        &self,
        backend: &SupportedBackend,
    ) -> Option<&BTreeMap<String, String>> {
        let Some(options_list) = self.backend_options.as_ref() else {
            return None;
        };

        for options in options_list {
            if let Ok(defaults_backend) = SupportedBackend::from_str(&options.0) {
                if &defaults_backend == backend {
                    return Some(&options.1);
                }
            }
        }

        None
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RepositoryOptions {
    pub name: String,
    pub url: String,
    pub password: Option<String>,
    pub initialise: bool,
    pub backend_options: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectorOptions {
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}
