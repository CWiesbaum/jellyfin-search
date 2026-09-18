# Phase 0 Research: Multi-Architecture Docker Image Publishing

## 1. How to produce the x86_64 and ARM64 binaries the image will contain

**Decision**: Cross-compile static musl binaries *outside* Docker using `cargo-zigbuild`
(the same tool the `.devcontainer/features/rust-cross` Feature already provisions) for the
`x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` targets. The Dockerfile then
only `COPY`s the correct prebuilt binary for the platform being built — no Rust compilation
ever happens inside a container.

**Rationale**: This repository's own history (commit `32ad27b`) already discovered and
documented that the `cross` tool's Docker+QEMU approach segfaults trying to emulate `rustc`
itself for this host/target architecture pairing, and adopted `cargo-zigbuild` + Zig
specifically to get reliable cross-compilation without that failure mode. Reusing that same,
already-verified path in CI avoids reintroducing a known-broken approach and requires no new
tooling decision.

**Alternatives considered**:
- `docker buildx build --platform linux/amd64,linux/arm64` compiling `cargo build --release`
  inside each per-arch container via QEMU — rejected: this is exactly the Docker+QEMU
  emulation-of-rustc approach this repo already found broken.
- The `cross` tool directly — rejected for the same documented reason.
- Native ARM64-hosted CI runners to compile without emulation — technically viable, but adds
  a second runner OS/toolchain-install path to maintain for no benefit over the
  already-working `cargo-zigbuild` path; rejected on Simplicity/YAGNI grounds.

## 2. How to assemble one multi-architecture image from prebuilt binaries

**Decision**: A single Dockerfile using `ARG TARGETARCH`, `FROM alpine:<pinned>`, and
`COPY dist/${TARGETARCH}/jellyfin-catalog-export /usr/local/bin/jellyfin-catalog-export`,
built with one `docker buildx build --platform linux/amd64,linux/arm64 --push` invocation.
Buildx invokes the Dockerfile once per requested platform, setting `TARGETARCH` to `amd64`
or `arm64` respectively for each; since only the tiny final `COPY` step differs per
platform (no compilation), this is a fast, emulation-free operation.

**Rationale**: Keeps a single Dockerfile and a single build invocation while still
producing one proper multi-architecture manifest list, without hand-assembling manifests.

**Alternatives considered**:
- Separate single-arch `docker build` calls followed by `docker manifest create`/
  `docker manifest push` to combine them — more moving parts for no benefit once binaries
  are pre-built outside Docker; buildx's native multi-platform support already does this.

## 3. CI trigger, version/tag validation, and atomicity

**Decision**: A GitHub Actions workflow (`.github/workflows/docker-publish.yml`) triggered
on push of tags matching `v*.*.*`, with a `!v*-*` exclusion pattern so any tag containing a
hyphen (e.g. a pre-release tag like `v0.2.0-beta.1`) never triggers the workflow — closing
the spec's Edge Case about `latest` never landing on an untested build. Before building, a
pre-flight step parses `Cargo.toml`'s
`version` field and fails the workflow if it doesn't match the pushed tag (stripped of its
leading `v`). Another pre-flight step checks Docker Hub (e.g. via
`docker manifest inspect cwiesbaum/jellyfin-catalog-export:<version>`, expecting a
"not found" result) and fails the workflow if that version tag already exists. Both target
architectures are then smoke-tested individually, and only after both pass does a single
`docker buildx build --platform linux/amd64,linux/arm64 --push` publish the version tag and
update `latest`.

**Rationale**: Ties publishing directly to the same semantic version already governed by the
project's constitution (Principle V), and turns the spec's "don't overwrite an existing
tag" and "no partial multi-arch publish" edge cases into concrete, fail-closed workflow
gates rather than relying on implicit registry behavior. A single `--push` invocation for
both platforms is atomic by construction: if either platform's context (its prebuilt binary
or smoke test) isn't ready, the build step never reaches `--push` for either.

**Alternatives considered**:
- Trigger on a GitHub "release published" event instead of a tag push — functionally
  similar, but couples publishing to release notes being filled in; a tag push more directly
  matches "a new version was released" without that extra dependency.

## 4. Base image pinning

**Decision**: Pin the Alpine base image to a specific minor-version tag (e.g. `alpine:3.20`)
rather than `alpine:latest`.

**Rationale**: Reproducible builds — `alpine:latest` can change the runtime libc/tooling
under a previously-published image tag without any change on this repo's side.

**Alternatives considered**:
- `alpine:latest` — rejected for reproducibility.
- Pinning by immutable content digest — more reproducible still, but harder to read/update
  by hand for the benefit gained; a minor-version tag is a reasonable middle ground
  consistent with Simplicity/YAGNI (Principle II).

## 5. Verifying both architectures run correctly before publishing

**Decision**: `scripts/docker-smoke-test.sh` builds/loads a single-platform image (via
`docker buildx build --platform <platform> --load`) and runs `--help` plus a full export
against the existing `wiremock`-backed fixture pattern already used in
`tests/integration/`, executed for `linux/amd64` natively and for `linux/arm64` under
QEMU user-mode emulation (registered once via `tonistiigi/binfmt`) of the already-compiled
static binary.

**Rationale**: Running a prebuilt static binary under QEMU is a well-supported, low-risk use
of emulation — unlike running `rustc` under it, which is the specific combination this repo
already found broken. This satisfies FR-004/SC-002 ("verified to run on both architectures
... before being made available") without needing physical ARM64 CI hardware.

**Alternatives considered**:
- Skip runtime verification and trust the build alone — rejected; doesn't satisfy SC-002.
- Use a real ARM64-hosted CI runner for verification — more faithful, but adds runner
  cost/complexity for a check emulation already satisfies adequately.
