#!/usr/bin/env bash
# Test-first for scripts/check-tag-not-published.sh (data-model.md's Image Tag validation
# rule; FR-007). Stubs `docker` on PATH to simulate both outcomes without touching real
# Docker Hub state.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FAKE_BIN_DIR="$(mktemp -d)"
trap 'rm -rf "$FAKE_BIN_DIR"' EXIT

# Case 1: tag already exists -> `docker manifest inspect` succeeds -> script must fail.
cat >"$FAKE_BIN_DIR/docker" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$FAKE_BIN_DIR/docker"

if PATH="$FAKE_BIN_DIR:$PATH" "$SCRIPT_DIR/check-tag-not-published.sh" 1.2.3; then
  echo "FAIL: expected non-zero exit when tag already exists" >&2
  exit 1
fi
echo "PASS: refuses an already-published tag"

# Case 2: tag does not exist -> `docker manifest inspect` fails -> script must succeed.
cat >"$FAKE_BIN_DIR/docker" <<'EOF'
#!/usr/bin/env bash
exit 1
EOF
chmod +x "$FAKE_BIN_DIR/docker"

if ! PATH="$FAKE_BIN_DIR:$PATH" "$SCRIPT_DIR/check-tag-not-published.sh" 1.2.3; then
  echo "FAIL: expected zero exit when tag does not exist" >&2
  exit 1
fi
echo "PASS: allows a not-yet-published tag"
