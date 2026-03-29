#!/bin/bash

set -euo pipefail

NEXT_VERSION=$1

sed -i "s/^version =.*$/version = \"$NEXT_VERSION\"/" Cargo.toml

docker build -t rust-prometheus-operator:local -f Containerfile .
cargo prepare
