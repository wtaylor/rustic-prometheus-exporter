# Rustic Prometheus Exporter

A Prometheus exporter for [Restic](https://restic.net/) repository metrics including repository size and last successful backup timestamp. Supports all backends currently [supported by Rustic](https://rustic.cli.rs/docs/comparison-restic.html#supported-storage-backends). Can be configured to scrape multiple repositories concurrently.

🚧 This project is currently pre-alpha software. Use at your own risk and raise issues when encountered. 🚧

![GitHub Release](https://img.shields.io/github/v/release/wtaylor/rustic-prometheus-exporter?style=for-the-badge)
![GitHub Release Date](https://img.shields.io/github/release-date/wtaylor/rustic-prometheus-exporter?style=for-the-badge)
![GitHub Repo stars](https://img.shields.io/github/stars/wtaylor/rustic-prometheus-exporter?style=for-the-badge)

## Usage

Binaries are built for Linux(gnu/musl), Windows and macOS(arm/intel) and are available from the releases page.

```bash
rustic-prometheus-exporter -c path/to/config.yaml
```

### Docker

Docker images are available for Linux arm and amd64 and are in the [GHCR](https://github.com/wtaylor/rustic-prometheus-exporter/pkgs/container/rustic-prometheus-exporter)

```bash
docker run -d -p 8080:8080 -v path/to/config.yaml:/etc/rustic-prometheus-exporter/config.yaml ghcr.io/wtaylor/rustic-prometheus-exporter:latest
```

## Configuration

All configuration is managed through a single config.yaml. Here's a documented sample config attempting to utilise every option available.

```yaml
# The http table contains all options for exposing the exporter endpoint 
http:
  # The listen local address for the exporter endpoint
  # Here, metrics will be available under http://localhost:8080/metrics
  listen: 0.0.0.0:8080
# General settings for the collector
collector:
  # Interval between collector runs, expect metrics to be scraped every 1 hour here
  interval: 1h
# Configuration of the Restic repositories
restic:
  # Cache directory for repository metadata, setting cache will speed up subsequent collections
  cache_dir: /tmp/rpe-cache
  # Set defaults to avoid repetition of common repository settings
  defaults:
    # The default password of any repository configured below, is overridden by supplying
    # a password to the repository object
    password: restic
    # Default backend_options supplies the defaults for a specific backend protocol 
    backend_options:
      # The REST backend protocol requires a username and password,
      # this is equivalent to the env vars RESTIC_REST_USERNAME/PASSWORD for Restic
      rest:
        username: restic
        password: restic
  # Your list of repositories to monitor
  repositories:
    # Name is used logging but also gets added as a Prometheus label
  - name: my-repo
    # The repository URL with protocol included (except for local paths)
    url: /tmp/my-repo
  - name: my-repo2
    url: /tmp/my-repo-2
    # This repository overrides restic.defaults.password with it's own password
    password: super-secret
    # This repository will get it's backend_options from defaults.backend_options.rest
  - name: my-remote-repo
    # The bit before the ':' is the key for defaults.backend_options.<key>
    # The only exception is a protocol-less local repo, for which you can use 'local' as the key
    url: rest://my.repo.local:8081/my-remote-repo
    # This repository partially overrides the defaults.backend_options.rest with a custom password
  - name: my-remote-repo2
    url: rest://my.repo.local:8081/my-remote-repo
    backend_options:
      password: my-super-special-password
```

---

No AI was used in the making of this project, it was all poorly written by hand.
