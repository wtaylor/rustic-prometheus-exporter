use std::collections::BTreeMap;

use reqwest::Url;
use rustic_backend::{SupportedBackend, util::location_to_type_and_path};
use rustic_core::{Credentials, Repository};
use tracing::info;

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
    let (backend_protocol, path) = location_to_type_and_path(&repository_options.url).unwrap();
    let mut location = repository_options.url.clone();
    info!("Repository is a {}", backend_protocol.to_string());

    let default_backend_options = app_options
        .restic
        .defaults
        .as_ref()
        .and_then(|d| d.get_options_for_backend(&backend_protocol));

    let mut backend_options = match default_backend_options {
        Some(defaults) => defaults.clone(),
        None => BTreeMap::new(),
    };

    if let Some(repo_backend_options) = repository_options.backend_options.clone() {
        backend_options.extend(repo_backend_options);
    }

    if backend_protocol == SupportedBackend::Rest {
        let mut location_url = Url::parse(&path).unwrap();
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

        location = format!("rest:{}", location_url);
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
