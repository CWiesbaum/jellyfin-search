#!/usr/bin/env bash
set -euo pipefail

# Options are exposed as uppercased env vars by the devcontainer CLI.
PYTHON_VERSION="${PYTHONVERSION:-3.12}"
SPECKIT_REPOSITORY="${SPECKITREPOSITORY:-https://github.com/github/spec-kit.git}"

# uv (from the ghcr.io/va-h/devcontainers-features/uv dependency) installs into
# the invoking user's home directory, so run it as the actual remote/container
# user rather than root - otherwise `specify` would only be reachable by root.
TARGET_USER="${_REMOTE_USER:-${_CONTAINER_USER:-root}}"

if ! su - "${TARGET_USER}" -c 'command -v uv' >/dev/null 2>&1; then
	echo "uv was not found for user '${TARGET_USER}'. The speckit Feature requires the uv Feature (ghcr.io/va-h/devcontainers-features/uv:1) to run first." >&2
	exit 1
fi

su - "${TARGET_USER}" -c "uv python install '${PYTHON_VERSION}' && uv tool install specify-cli --from 'git+${SPECKIT_REPOSITORY}'"
