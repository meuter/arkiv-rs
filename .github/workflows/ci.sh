#/usr/bin/env bash

set -x

cargo build
cargo build --no-default-features
cargo build --all-features
cargo build --no-default-features -F bzip
cargo build --no-default-features -F gzip
cargo build --no-default-features -F tar
cargo build --no-default-features -F xz
cargo build --no-default-features -F zip

cargo clippy
cargo audit
