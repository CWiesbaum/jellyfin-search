#!/usr/bin/env bash
# Builds a single-platform image and verifies it runs correctly: `--help` succeeds, and a
# full export against the existing wiremock-backed mock Jellyfin server pattern (reused
# from tests/integration/) produces the expected output. See
# specs/003-docker-image-publish/research.md §5 and quickstart.md.
set -euo pipefail

PLATFORM="${1:?usage: docker-smoke-test.sh <platform> (e.g. linux/amd64 or linux/arm64)}"
IMAGE_TAG="jce:smoke-test"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

echo "==> building ${IMAGE_TAG} for ${PLATFORM}"
docker buildx build --platform "$PLATFORM" --load -t "$IMAGE_TAG" .

echo "==> checking --help"
# The native binary itself exits 1 on --help (see src/main.rs's try_parse() handling,
# specs/001-remote-movie-catalog) — the image mirrors that unchanged, so this checks for
# the expected usage text rather than a specific exit code. Output is captured into a
# variable (rather than piped straight to grep) so `set -o pipefail`'s status doesn't just
# reflect docker run's known-non-zero exit regardless of what grep finds.
help_output="$(docker run --rm "$IMAGE_TAG" --help 2>&1 || true)"
if ! grep -q "^Usage: jellyfin-catalog-export" <<<"$help_output"; then
  echo "error: --help did not produce the expected usage text for ${IMAGE_TAG} (${PLATFORM}); actual output was:" >&2
  echo "$help_output" >&2
  exit 1
fi

echo "==> running full export against a mock Jellyfin server"
JCE_DOCKER_SMOKE_IMAGE="$IMAGE_TAG" cargo test --test integration \
  docker_image_runs_full_export_against_mock_jellyfin_server -- --test-threads=1 --nocapture

echo "OK: ${IMAGE_TAG} (${PLATFORM}) passed the smoke test"
