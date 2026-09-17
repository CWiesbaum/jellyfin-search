#!/usr/bin/env bash
set -euo pipefail

# Options are exposed as uppercased env vars by the devcontainer CLI.
TARGETS="${TARGETS:-x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu,x86_64-unknown-linux-musl,aarch64-unknown-linux-musl}"

# The base Rust image installs rustup/cargo into a shared, world-writable
# location (not the invoking user's $HOME) and points root at it via the
# RUSTUP_HOME/CARGO_HOME env vars set in its Dockerfile. `su - <user>` starts
# a login shell that drops those Docker-set env vars, so re-export them
# explicitly - otherwise rustup falls back to the target user's empty
# $HOME/.rustup and fails with "no default toolchain".
RUSTUP_HOME="${RUSTUP_HOME:-/usr/local/rustup}"
CARGO_HOME="${CARGO_HOME:-/usr/local/cargo}"
TARGET_USER="${_REMOTE_USER:-${_CONTAINER_USER:-root}}"

run_as_user() {
	su - "${TARGET_USER}" -c "export RUSTUP_HOME='${RUSTUP_HOME}' CARGO_HOME='${CARGO_HOME}'; $1"
}

if ! run_as_user 'command -v cargo' >/dev/null 2>&1; then
	echo "cargo was not found for user '${TARGET_USER}'. The rust-cross Feature requires a Rust toolchain (rustup/cargo) to already be installed." >&2
	exit 1
fi

# Zig is what cargo-zigbuild actually drives to link/cross-compile (it bundles
# clang plus prebuilt sysroots, including musl, for every target pairing).
# Installed via pip's official `ziglang` package rather than a downloaded
# tarball: cargo-zigbuild auto-detects it and shells out as `python3 -m
# ziglang` when no standalone `zig` binary is on PATH (see
# https://pypi.org/project/ziglang/). `file` is installed to let callers
# verify the architecture of cross-compiled binaries.
apt-get update
apt-get install -y --no-install-recommends python3-pip file
rm -rf /var/lib/apt/lists/*

# Debian bookworm/trixie mark the system Python as "externally managed"
# (PEP 668); bullseye's older pip predates that and doesn't understand the
# flag at all, hence the fallback.
python3 -m pip install --break-system-packages ziglang 2>/dev/null || python3 -m pip install ziglang

run_as_user "cargo install --locked cargo-zigbuild"

IFS=',' read -ra TARGET_LIST <<< "${TARGETS}"
for TARGET in "${TARGET_LIST[@]}"; do
	run_as_user "rustup target add ${TARGET}"
done

# `cargo xbuild --target <triple> [--release]` is the one concise, repeatable
# cross-compilation command this Feature sets up - cargo-zigbuild handles the
# linker/sysroot wiring, no per-target [target.*] linker config needed in the
# consuming project itself. Written to $CARGO_HOME/config.toml (not
# ~/.cargo/config.toml) since CARGO_HOME here is the shared location above,
# not the target user's home - and that directory is already world-writable,
# so this can be done directly as root without another su hop.
if ! grep -q "xbuild" "${CARGO_HOME}/config.toml" 2>/dev/null; then
	printf '\n[alias]\nxbuild = "zigbuild"\n' >> "${CARGO_HOME}/config.toml"
fi
