#!/usr/bin/env bash
# Builds and pushes the multi-architecture manifest under both the pinned version tag and
# `latest`, in one atomic buildx invocation (FR-006, FR-008). Assumes the caller has
# already `docker login`'d and that dist/amd64 and dist/arm64 are populated (see
# scripts/build-cross-binaries.sh).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO="cwiesbaum/jellyfin-catalog-export"

VERSION="$("$SCRIPT_DIR/current-version.sh")"
"$SCRIPT_DIR/check-tag-not-published.sh" "$VERSION"

docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t "${REPO}:${VERSION}" \
  -t "${REPO}:latest" \
  --push \
  "$REPO_ROOT"

echo "OK: published ${REPO}:${VERSION} and ${REPO}:latest"
