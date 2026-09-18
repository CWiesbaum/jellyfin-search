#!/usr/bin/env bash
# Cross-compiles the jellyfin-catalog-export binary for both Docker image
# architectures and stages the results where the Dockerfile's `ARG TARGETARCH`
# expects them. See specs/003-docker-image-publish/research.md §1-2.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

cargo zigbuild --release --target x86_64-unknown-linux-musl
cargo zigbuild --release --target aarch64-unknown-linux-musl

mkdir -p dist/amd64 dist/arm64
cp target/x86_64-unknown-linux-musl/release/jellyfin-catalog-export dist/amd64/jellyfin-catalog-export
cp target/aarch64-unknown-linux-musl/release/jellyfin-catalog-export dist/arm64/jellyfin-catalog-export

echo "OK: staged dist/amd64/jellyfin-catalog-export and dist/arm64/jellyfin-catalog-export"
