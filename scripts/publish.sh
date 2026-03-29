#!/bin/bash

RELEASE_VERSION=$1

docker image tag rust-prometheus-operator:local ghcr.io/wtaylor/rust-prometheus-operator:latest
docker image tag rust-prometheus-operator:local ghcr.io/wtaylor/rust-prometheus-operator:$RELEASE_VERSION

docker push -a ghcr.io/wtaylor/rust-prometheus-operator
cargo publish
