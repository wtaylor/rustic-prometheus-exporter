use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppOptions {
    pub http: HttpOptions,
    pub restic: ResticOptions,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpOptions {
    pub listen: SocketAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResticOptions {
    pub rpe_cache: Option<PathBuf>,
    pub default_target_credentials: Option<DefaultTargetCredentialsOptions>,
    pub repositories: Vec<RepositoryOptions>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RepositoryOptions {
    pub url: String,
    pub password: String,
    pub target_credentials: Option<CredentialOptions>,
    pub name: String,
    pub additional_labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultTargetCredentialsOptions {
    pub rest: Option<RestCredentialOptions>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum CredentialOptions {
    Rest(RestCredentialOptions),
}

#[derive(Debug, Deserialize, Clone)]
pub struct RestCredentialOptions {
    pub username: String,
    pub password: String,
}
