use reqwest::Url;
use rustic_backend::{SupportedBackend, util::location_to_type_and_path};
use rustic_core::{Credentials, Repository};

use crate::options::{AppOptions, RepositoryOptions};

pub fn get_credentials(
    app_options: &AppOptions,
    repository_options: &RepositoryOptions,
) -> Credentials {
    let password = app_options
        .restic
        .defaults
        .as_ref()
        .and_then(|d| d.password.as_ref())
        .or(repository_options.password.as_ref())
        .unwrap();
    Credentials::password(password)
}

pub fn get_repository(
    app_options: &AppOptions,
    repository_options: &RepositoryOptions,
) -> Repository<()> {
    let (backend_protocol, location) = location_to_type_and_path(&repository_options.url).unwrap();
    let mut location = location.to_string();
    let backend_protocol_str = backend_protocol.to_string();

    let mut backend_options = app_options
        .restic
        .defaults
        .as_ref()
        .and_then(|d| d.backend_options.as_ref())
        .and_then(|d| d.get(&backend_protocol_str))
        .and_then(|d| Some(d.clone()))
        .unwrap_or_default();

    if let Some(repo_backend_options) = repository_options.backend_options.clone() {
        backend_options.extend(repo_backend_options);
    }

    if backend_protocol == SupportedBackend::Rest {
        let mut location_url = Url::parse(&location).unwrap();
        if location_url.username() == "" {
            if let Some(username) = backend_options.get("username") {
                location_url.set_username(username).unwrap();
            }
        }

        if location_url.password().is_none() {
            if let Some(password) = backend_options.get("password") {
                location_url.set_password(Some(password)).unwrap();
            }
        }

        location = location_url.to_string();
    }

    let backend = rustic_backend::BackendOptions::default()
        .repository(&location)
        .options(backend_options)
        .to_backends()
        .unwrap();

    let mut repo_options = rustic_core::RepositoryOptions::default();
    match app_options.restic.cache_dir.as_deref() {
        Some(value) => {
            repo_options = repo_options.cache_dir(value);
        }
        None => {
            repo_options = repo_options.no_cache(true);
        }
    }

    Repository::new(&repo_options, &backend).unwrap()
}
