#!/usr/bin/env bash
# Prints Cargo.toml's package version — the single source of truth for the pinned image
# tag (data-model.md's Release Version entity; FR-006).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_TOML="$SCRIPT_DIR/../Cargo.toml"

grep -m1 '^version' "$CARGO_TOML" | sed -E 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/'
