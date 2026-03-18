pub struct AppOptions {
    http: HttpOptions,
    restic: ResticOptions,
}

pub struct HttpOptions {
    listen: String,
}

pub struct ResticOptions {
    repositories: Vec<RepositoryOptions>,
}

pub struct RepositoryOptions {
    url: String,
    credential: CredentialOptions,
}

pub struct CredentialOptions {}
