#!/usr/bin/env bash
# Fails if the given git tag doesn't match Cargo.toml's version — the CI workflow's
# pre-flight gate against publishing a mislabeled image (data-model.md's Release Version
# validation rule).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_TAG="${1:?usage: check-version-matches-tag.sh <git-tag>}"
EXPECTED="${GIT_TAG#v}"
ACTUAL="$("$SCRIPT_DIR/current-version.sh")"

if [ "$EXPECTED" != "$ACTUAL" ]; then
  echo "error: git tag '${GIT_TAG}' (version '${EXPECTED}') does not match Cargo.toml version '${ACTUAL}'" >&2
  exit 1
fi

echo "OK: git tag ${GIT_TAG} matches Cargo.toml version ${ACTUAL}"
