#!/usr/bin/env bash
# Test-first for scripts/check-version-matches-tag.sh (data-model.md's Release Version
# validation rule). Checks against the real Cargo.toml version rather than stubbing
# anything, since current-version.sh has no external dependency to fake.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACTUAL="$("$SCRIPT_DIR/current-version.sh")"

if ! "$SCRIPT_DIR/check-version-matches-tag.sh" "v${ACTUAL}"; then
  echo "FAIL: expected success when tag matches Cargo.toml version" >&2
  exit 1
fi
echo "PASS: accepts a matching tag"

if "$SCRIPT_DIR/check-version-matches-tag.sh" "v${ACTUAL}-does-not-exist"; then
  echo "FAIL: expected failure when tag does not match Cargo.toml version" >&2
  exit 1
fi
echo "PASS: rejects a mismatched tag"
