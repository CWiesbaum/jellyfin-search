#!/usr/bin/env bash
# Asserts README.md links back to the GitHub repository (FR-004; data-model.md's Docker Hub
# Repository Metadata validation rule), so the Docker Hub overview page — sourced from this
# same README — gives visitors a way back to the source.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
README="$SCRIPT_DIR/../README.md"
GITHUB_URL="https://github.com/CWiesbaum/jellyfin-search"

if ! grep -qF "$GITHUB_URL" "$README"; then
  echo "error: README.md does not contain a link to ${GITHUB_URL}" >&2
  exit 1
fi

echo "OK: README.md links to ${GITHUB_URL}"
