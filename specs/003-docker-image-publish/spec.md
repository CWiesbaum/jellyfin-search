# Feature Specification: Multi-Architecture Docker Image Publishing

**Feature Branch**: `003-docker-image-publish`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "I want to add a Dockerfile for this repository. This Dockerfile should be based on alpine and include the binary. The Dockerfile should be build for ARM and X86 and pushed to dockerhub."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run the exporter via Docker on any supported host (Priority: P1)

An operator who wants to run the Jellyfin catalog exporter does not want to install Rust or
a build toolchain. They want to pull a ready-to-run container image and run it directly on
whatever machine they have, whether it's a typical x86_64 server/desktop or an ARM64 device
(e.g. a Raspberry Pi or ARM-based NAS/cloud instance).

**Why this priority**: This is the core ask — a published, ready-to-run image is the entire
point of the feature. Without it, nothing else in this feature has value.

**Independent Test**: Can be fully tested by pulling the published image on an x86_64 host
and separately on an ARM64 host, running it with `--help` and with a real export against a
test Jellyfin server, and confirming both produce the same behavior as running the native
binary directly.

**Acceptance Scenarios**:

1. **Given** a published image tag, **When** an operator on an x86_64 host runs
   `docker pull` followed by `docker run`, **Then** the tool starts and behaves identically
   to the natively-built binary.
2. **Given** the same published image tag, **When** an operator on an ARM64 host runs the
   same `docker pull`/`docker run` commands, **Then** the tool starts and behaves
   identically to the natively-built binary.
3. **Given** the published image, **When** its contents are inspected, **Then** its base
   layer is Alpine Linux and it contains no leftover build toolchain, source code, or
   intermediate build artifacts.

---

### User Story 2 - Discover and pin to a specific released version (Priority: P2)

A consumer browsing Docker Hub wants to know which version of the tool a given image tag
corresponds to, and wants the option to pin their deployment to an exact version rather than
always tracking the newest build.

**Why this priority**: Without predictable tagging, consumers can't safely automate
deployments or roll back, which undermines trust in the published artifact even if the image
itself works.

**Independent Test**: Can be fully tested by publishing two different versions of the tool
and confirming each is retrievable by its own distinct tag, and that a `latest`-style tag
always resolves to the newest stable release.

**Acceptance Scenarios**:

1. **Given** a released version of the tool, **When** its image is published, **Then** it is
   retrievable by a tag that uniquely identifies that version.
2. **Given** multiple published versions, **When** a consumer pulls the "latest" tag,
   **Then** they receive the newest stable released version.

---

### User Story 3 - Maintainer publishes a new image for a new release (Priority: P3)

When the maintainer cuts a new release of the tool, they need the corresponding container
image built for both supported architectures and published to Docker Hub as part of that
release, without hand-building images on multiple machines.

**Why this priority**: This is about the maintainer's workflow rather than end-user value
directly, but it's what keeps User Stories 1 and 2 true for every future release rather than
just a one-time manual publish.

**Independent Test**: Can be fully tested by cutting a release and confirming a
correctly-tagged, multi-architecture image appears on Docker Hub without additional manual
image-building steps beyond what the release process already requires.

**Acceptance Scenarios**:

1. **Given** a new version is released, **When** the publish process runs, **Then** images
   for both x86_64 and ARM64 are built, verified to run, and published under that version's
   tag and the "latest" tag (for stable releases).

---

### Edge Cases

- What happens if the image build succeeds for one architecture but fails for the other? The
  publish MUST NOT go out as a partial multi-architecture manifest — either both
  architectures publish successfully or neither does.
- What happens if a publish is attempted for a version tag that already exists on Docker
  Hub? The system MUST NOT silently overwrite a previously published, immutable version tag.
- How are in-progress/pre-release builds distinguished from stable releases, so consumers
  pulling `latest` never land on an untested build?
- What happens if Docker Hub is unreachable or rejects the credentials at publish time? The
  failure MUST be surfaced clearly rather than reported as a successful publish.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The repository MUST provide a Dockerfile that packages the
  `jellyfin-catalog-export` binary into a container image.
- **FR-002**: The final runtime layer of the built image MUST be based on Alpine Linux.
- **FR-003**: The image MUST be built for both x86_64 and ARM64 (aarch64) architectures and
  published under a single multi-architecture tag/manifest, so a consumer's `docker pull`
  automatically resolves to the correct architecture for their host.
- **FR-004**: The tool MUST run correctly inside the container on both architectures,
  producing behavior equivalent to running the native binary directly.
- **FR-005**: The build MUST NOT leave build tooling, compilers, source code, or
  intermediate build artifacts in the final published image layer.
- **FR-006**: Published images MUST be tagged so that a consumer can pin to an exact
  released version (a tag uniquely identifying that version) as well as track the newest
  stable release via a floating "latest"-style tag.
- **FR-007**: The system MUST NOT overwrite a previously published, immutable version tag
  with different image contents.
- **FR-008**: A multi-architecture publish MUST be all-or-nothing: if either architecture's
  build fails or fails verification, the publish for that version MUST NOT proceed for
  either architecture.
- **FR-009**: The image MUST be published to Docker Hub under the `cwiesbaum` namespace, in
  a repository named `jellyfin-catalog-export` (i.e. `cwiesbaum/jellyfin-catalog-export`).
- **FR-010**: The build-and-publish process MUST run automatically (via CI) whenever a new
  version of the tool is tagged/released, without requiring the maintainer to manually
  build or push images.
- **FR-011**: The README MUST be updated with instructions for pulling and running the
  published image, alongside the existing "build from source" instructions.

### Key Entities

- **Container Image**: A published, runnable packaging of the CLI binary for a given
  version. Has an architecture-agnostic tag (multi-arch manifest) resolving to one
  architecture-specific image per supported architecture (x86_64, ARM64).
- **Image Tag**: An identifier consumers use to pull a specific image — either pinned to an
  exact released version, or a floating tag that always resolves to the current latest
  stable release.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator with Docker already installed can go from reading the README to
  successfully running the exporter in under 5 minutes, without installing a Rust toolchain,
  on either an x86_64 or an ARM64 host.
- **SC-002**: 100% of published image tags run the tool successfully on both x86_64 and
  ARM64 hosts before being made available to consumers.
- **SC-003**: Every published image tag can be traced back to exactly one released version
  of the tool, with no ambiguity about what code it contains.
- **SC-004**: Publishing a new release's image requires no more manual, per-architecture
  build steps than publishing today's source-only release already requires.

## Assumptions

- "ARM" refers to 64-bit ARM (ARM64/aarch64) — the architecture this repository's existing
  cross-compilation tooling (the `rust-cross` devcontainer feature) already targets — not
  32-bit ARM (e.g. armv7).
- "X86" refers to x86_64 (amd64), matching this repository's existing
  `x86_64-unknown-linux-musl` cross-compilation target.
- The binary is built against a musl target (as this repository's cross-compilation tooling
  already supports) so it runs on Alpine's musl libc without extra runtime dependencies.
- Alpine is required only for the final runtime image; earlier build stages may use a
  different base image if needed, consistent with standard multi-stage Docker build
  practice.
- A Docker Hub account under the `cwiesbaum` namespace already exists, and credentials for
  it will be made available to CI as secrets so the automated publish process can
  authenticate; provisioning a new Docker Hub account is out of scope.
- Version tags for the image are derived from the version already declared in the project's
  `Cargo.toml`; this feature does not introduce a separate versioning scheme.
- This feature does not cover publishing to any registry other than Docker Hub (e.g. GitHub
  Container Registry), and does not cover image signing/provenance attestation.
