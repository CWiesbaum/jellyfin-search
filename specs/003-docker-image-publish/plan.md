# Implementation Plan: Multi-Architecture Docker Image Publishing

**Branch**: `003-docker-image-publish` | **Date**: 2026-09-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-docker-image-publish/spec.md`

## Summary

Package the existing `jellyfin-catalog-export` CLI binary into a multi-architecture
(x86_64 + ARM64) Docker image with an Alpine Linux runtime layer, and publish it to Docker
Hub under `cwiesbaum/jellyfin-catalog-export` automatically via CI whenever a version is
tagged/released. The binaries for both architectures are cross-compiled *outside* Docker
using the `cargo-zigbuild`/Zig toolchain this repository's devcontainer already provisions
(see `32ad27b`), and the Dockerfile only copies the correct prebuilt static binary per
target platform — no compilation happens inside a QEMU-emulated container, sidestepping the
rustc-under-QEMU segfault this repo already hit and worked around once. A single
`docker buildx build --platform linux/amd64,linux/arm64 --push` invocation publishes both
architectures under one tag atomically, and the CI workflow verifies the version isn't
already published and that both architectures run correctly before publishing.

## Technical Context

**Language/Version**: Rust (stable toolchain, 2024 edition) — unchanged. This feature adds
no new Rust code; it packages the existing compiled binary.

**Primary Dependencies**: Docker (Dockerfile + Buildx for multi-platform manifests),
`cargo-zigbuild` + Zig (already provisioned by `.devcontainer/features/rust-cross`, reused
here to cross-compile both target triples in CI), GitHub Actions (CI trigger, build, and
publish), Alpine Linux (pinned minor-version tag, e.g. `alpine:3.20`, as the runtime base).

**Storage**: N/A.

**Testing**: No new `cargo test` cases (no Rust logic changes). A new
`scripts/docker-smoke-test.sh` validates the built image: runs `--help` and a full export
against the existing `wiremock`-backed fixture approach (reusing the pattern from
`tests/integration/`) inside the container for each target platform (ARM64 verified via
QEMU user-mode emulation of the *already-compiled* static binary — not of `rustc` — which is
a well-supported use of emulation, unlike the compilation-under-emulation path this repo
already rejected). This script is written first, expected to fail (no image exists yet),
then made to pass once the Dockerfile and cross-compiled binaries exist — the Test-First
principle applied to this feature's actual deliverable (an image), not to Rust unit tests
that don't apply here.

**Target Platform**: Linux containers on x86_64 and ARM64 hosts (Docker Hub image); build
executes on GitHub Actions' Linux x86_64 runners cross-compiling for both targets.

**Project Type**: Single project (unchanged) — this feature adds packaging/CI artifacts
(Dockerfile, CI workflow, a smoke-test script) at the repository root and under
`.github/workflows/`, and touches no existing `src/` or `tests/` code.

**Performance Goals**: N/A for the tool itself (unchanged runtime behavior). CI budget: the
full build-verify-publish workflow should complete in a time comparable to a normal
`cargo test` + release-build CI run (no explicit SLA requested; kept unspecified by design
per the spec's Assumptions — this is an internal CI concern, not a user-facing outcome).

**Constraints**:
- MUST NOT compile Rust code inside a Docker/QEMU-emulated container — this repository's own
  history (`32ad27b`) documents that `cross`'s Docker+QEMU approach segfaults trying to
  emulate `rustc` for the aarch64 target on this host; `cargo-zigbuild` was adopted
  specifically to avoid that, and this feature reuses the same approach for CI rather than
  reintroducing the failure mode via `docker buildx build --platform ... ` compiling in
  place.
- Final published image layer MUST contain no compiler, source code, or intermediate build
  artifacts (FR-005; constitution Quality Standards — no dead weight in shipped artifacts).
- Version tags MUST be immutable once published (FR-007) and MUST match the version already
  declared in `Cargo.toml` (spec Assumptions + constitution Principle V, Semantic
  Versioning) — the workflow must fail closed on a tag/`Cargo.toml` mismatch rather than
  publish a mislabeled image.
- A multi-architecture publish MUST be atomic — a failure or verification failure on either
  architecture MUST prevent publishing for both (FR-008).

**Scale/Scope**: One Dockerfile, one `.dockerignore`, one GitHub Actions workflow file, six
scripts (`build-cross-binaries.sh`, `docker-smoke-test.sh`, `current-version.sh`,
`check-tag-not-published.sh`, `check-version-matches-tag.sh`, `publish-image.sh`) plus their
test scripts, and a README update. No new application modules.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Test-First (NON-NEGOTIABLE)** — PASS. Each behavioral script is preceded by its own
  failing test: `scripts/docker-smoke-test.sh` before the Dockerfile (User Story 1),
  `scripts/test-check-tag-not-published.sh` before `check-tag-not-published.sh` (User
  Story 2), and `scripts/test-check-version-matches-tag.sh` before
  `check-version-matches-tag.sh` (User Story 3) — the same red-green discipline as Rust unit
  tests, applied uniformly across all three stories' actual deliverables, not just the
  runtime-behavior story.
- **II. Simplicity & YAGNI** — PASS. One Dockerfile, one workflow file, one smoke-test
  script — no reusable-workflow abstraction, no build-matrix beyond the two architectures
  the spec actually requires, no registry beyond Docker Hub (spec Assumptions explicitly
  scope out GHCR/signing).
- **III. Code Review Discipline** — PASS (process-level; no plan-time design impact).
- **IV. Observability** — PASS by design: the CI workflow's pre-flight checks (version/tag
  match, existing-tag check) and the smoke test are required to fail loudly with a clear
  message rather than silently skip or report false success (spec Edge Cases).
- **V. Semantic Versioning & Breaking Changes** — PASS. Image version tags are derived
  directly from `Cargo.toml`'s existing semver version (no parallel versioning scheme
  introduced), and the workflow enforces that the triggering git tag and `Cargo.toml` agree
  before publishing.

No violations identified. Complexity Tracking table is not needed.

**Post-Design Re-check** (after Phase 1): The data model (Container Image / Image Tag /
Release Version) and the Docker image contract introduce no new dependencies, services, or
abstractions beyond what is listed above. All gates remain PASS.

## Project Structure

### Documentation (this feature)

```text
specs/003-docker-image-publish/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/
│   └── docker-image-contract.md   # Phase 1 output — the image's external interface
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
Dockerfile                          # NEW: single-stage, FROM alpine:<pinned>, ARG
                                     # TARGETARCH selects which prebuilt binary to COPY in.
.dockerignore                       # NEW: keeps build context to just the prebuilt
                                     # dist/ binaries (excludes src/, target/, .git/, etc.)

.github/
└── workflows/
    └── docker-publish.yml          # NEW: triggered on tags matching v*.*.*; cross-compiles
                                     # both targets via cargo-zigbuild, verifies
                                     # tag == Cargo.toml version and that the tag doesn't
                                     # already exist on Docker Hub, runs the smoke test per
                                     # architecture, then `docker buildx build --push`s one
                                     # multi-arch manifest tagged `<version>` and `latest`.

scripts/
├── build-cross-binaries.sh         # NEW: cross-compiles both target triples via
│                                    # cargo-zigbuild and populates dist/<arch>/.
├── docker-smoke-test.sh            # NEW: builds/loads a single-platform image and runs
│                                    # `--help` plus a full export against the existing
│                                    # wiremock-backed fixture pattern inside the container.
├── current-version.sh              # NEW: prints Cargo.toml's package version.
├── check-tag-not-published.sh      # NEW: fails if a version tag already exists on
│                                    # Docker Hub.
├── test-check-tag-not-published.sh # NEW: stubs `docker` to verify the above script's
│                                    # exit codes in both the "exists" and "not found" cases.
├── check-version-matches-tag.sh    # NEW: fails if the pushed git tag doesn't match
│                                    # Cargo.toml's version.
├── test-check-version-matches-tag.sh # NEW: verifies the above script's match/mismatch
│                                    # exit codes.
└── publish-image.sh                # NEW: builds+pushes the multi-arch manifest under
                                     # both the pinned version tag and `latest`.

README.md                           # MODIFIED: add "Run via Docker" section (FR-011)
                                     # alongside the existing "Build"/"Run" instructions.
```

Every existing file under `src/` and `tests/` is unchanged by this feature — it is purely
additive packaging/CI, consistent with `001-remote-movie-catalog` and
`002-ascii-terminal-theme` being untouched by it.

**Structure Decision**: Option 1 (single project), unchanged. This feature adds
packaging/CI artifacts alongside the existing single Rust CLI project rather than
introducing a new component or service.

## Complexity Tracking

*No Constitution Check violations were identified — this table is intentionally empty.*
