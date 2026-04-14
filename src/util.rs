use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use reqwest::Url;
use rustic_backend::{SupportedBackend, util::location_to_type_and_path};
use rustic_core::{Credentials, Repository};

use crate::options::{AppOptions, RepositoryOptions};

pub fn get_credentials(
    app_options: &AppOptions,
    repository_options: &RepositoryOptions,
) -> Option<Credentials> {
    app_options
        .restic
        .defaults
        .as_ref()
        .and_then(|d| d.password.as_ref())
        .or(repository_options.password.as_ref())
        .and_then(|password| Some(Credentials::password(password)))
}

pub fn get_repository(
    app_options: &AppOptions,
    repository_options: &RepositoryOptions,
) -> Result<Repository<()>> {
    let (backend_protocol, path) = location_to_type_and_path(&repository_options.url)?;
    let mut location = repository_options.url.clone();

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
        let mut location_url = Url::parse(&path)
            .with_context(|| "failed to parse url for rest backend, is it malformed?")?;
        if location_url.username() == "" {
            if let Some(username) = backend_options.get("username") {
                if location_url.set_username(username).is_err() {
                    bail!("failed to set username on rest backend url, is it malformed?");
                }
            }
        }

        if location_url.password().is_none() {
            if let Some(password) = backend_options.get("password") {
                if location_url.set_password(Some(password)).is_err() {
                    bail!("failed to set password on rest backend url, is it malformed?");
                }
            }
        }

        location = format!("rest:{}", location_url);
    }

    let backend = rustic_backend::BackendOptions::default()
        .repository(&location)
        .options(backend_options)
        .to_backends()
        .with_context(|| "failed to construct backend")?;

    let mut repo_options = rustic_core::RepositoryOptions::default();
    match app_options.restic.cache_dir.as_deref() {
        Some(value) => {
            repo_options = repo_options.cache_dir(value);
        }
        None => {
            repo_options = repo_options.no_cache(true);
        }
    }

    Ok(Repository::new(&repo_options, &backend)
        .with_context(|| "failed to construct repository from backend")?)
}
