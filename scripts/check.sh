#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo check --locked --no-default-features
for feature in artifact-physical-effects capability retention world-verifier world-policy-projection; do
    cargo check --locked --no-default-features --features "$feature"
done
