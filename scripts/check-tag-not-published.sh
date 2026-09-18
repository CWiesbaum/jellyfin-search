#!/usr/bin/env bash
# Fails if the given version tag already exists on Docker Hub — a pinned tag MUST NOT be
# reassigned once published (data-model.md's Image Tag validation rule; FR-007).
set -euo pipefail

VERSION="${1:?usage: check-tag-not-published.sh <version>}"
REPO="cwiesbaum/jellyfin-catalog-export"

if docker manifest inspect "${REPO}:${VERSION}" >/dev/null 2>&1; then
  echo "error: ${REPO}:${VERSION} already exists on Docker Hub - refusing to overwrite an immutable pinned tag" >&2
  exit 1
fi

echo "OK: ${REPO}:${VERSION} is not yet published"
