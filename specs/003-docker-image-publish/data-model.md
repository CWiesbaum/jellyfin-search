# Phase 1 Data Model: Multi-Architecture Docker Image Publishing

This feature introduces no application data or persistence. The "entities" below are the
release/publishing concepts the spec and CI workflow operate on, not stored records.

## Release Version

The single source of truth for what version is being packaged.

| Field     | Description                                                         |
|-----------|----------------------------------------------------------------------|
| `version` | Semver string, read from `Cargo.toml`'s `[package].version`.        |
| `git_tag` | The pushed git tag that triggers publishing, expected as `v<version>`. |

**Validation rule**: `git_tag` (with its leading `v` stripped) MUST equal `version` exactly,
or the publish workflow fails before any build step runs (FR-007's spirit extended to
prevent a *mislabeled* publish, not just an overwritten one).

**Validation rule**: `git_tag` MUST NOT contain a hyphen (a pre-release suffix, e.g.
`-beta.1`) — such tags are excluded from the CI trigger entirely (research.md §3's `!v*-*`
exclusion), so the `latest` Image Tag never resolves to an untested, non-stable build (spec
Edge Cases).

## Container Image

A published, runnable packaging of the CLI binary for one Release Version.

| Field           | Description                                                              |
|-----------------|---------------------------------------------------------------------------|
| `repository`    | Fixed: `cwiesbaum/jellyfin-catalog-export` (FR-009).                     |
| `architectures` | Fixed set: `linux/amd64`, `linux/arm64` (FR-003).                       |
| `source_version`| The Release Version this image was built from.                          |
| `base_image`    | Pinned Alpine version tag (e.g. `alpine:3.20`) — see research.md §4.     |

**State transitions**: `pending` (workflow triggered) → `verified` (both architectures pass
`scripts/docker-smoke-test.sh`) → `published` (manifest pushed under both tags) — a failure
at either of the first two states MUST NOT reach `published` for either architecture
(FR-008, atomicity).

## Image Tag

An identifier consumers use with `docker pull`/`docker run`.

| Field    | Type                              | Description                                          |
|----------|------------------------------------|-------------------------------------------------------|
| `value`  | `<version>` or `latest`            | Concrete tag string.                                  |
| `kind`   | `pinned` \| `floating`             | `pinned` tags are immutable once published (FR-007); `floating` (`latest`) is reassigned to point at each new stable Container Image. |
| `points_to` | Container Image                 | The manifest list this tag currently resolves to.     |

**Validation rule**: A `pinned` tag, once it has a `points_to` value, MUST NOT be
reassigned — a publish attempt targeting an already-existing pinned tag fails the workflow
instead (FR-007, spec Edge Cases).
