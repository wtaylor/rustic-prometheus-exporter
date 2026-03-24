use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppOptions {
    pub http: HttpOptions,
    pub restic: ResticOptions,
}

#[derive(Debug, Deserialize)]
pub struct HttpOptions {
    pub listen: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct ResticOptions {
    pub default_target_credentials: Option<DefaultTargetCredentialsOptions>,
    pub repositories: Vec<RepositoryOptions>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryOptions {
    pub url: String,
    pub password: String,
    pub target_credentials: Option<CredentialOptions>,
}

#[derive(Debug, Deserialize)]
pub struct DefaultTargetCredentialsOptions {
    pub rest: Option<RestCredentialOptions>,
}

#[derive(Debug, Deserialize)]
pub enum CredentialOptions {
    Rest(RestCredentialOptions),
}

#[derive(Debug, Deserialize)]
pub struct RestCredentialOptions {
    pub username: String,
    pub password: String,
}
