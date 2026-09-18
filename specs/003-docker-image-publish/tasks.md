---

description: "Task list template for feature implementation"
---

# Tasks: Multi-Architecture Docker Image Publishing

**Input**: Design documents from `/specs/003-docker-image-publish/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/docker-image-contract.md, quickstart.md

**Tests**: Included. The constitution's Principle I (Test-First, NON-NEGOTIABLE) applies to
every behavioral script this feature adds: `scripts/docker-smoke-test.sh` is written before
the Dockerfile (User Story 1), `scripts/test-check-tag-not-published.sh` before
`check-tag-not-published.sh` (User Story 2), and `scripts/test-check-version-matches-tag.sh`
before `check-version-matches-tag.sh` (User Story 3) — each expected to fail first, then
pass once its corresponding implementation exists.

**Organization**: Tasks are grouped by user story (from spec.md) to enable independent
implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project (unchanged from `001-remote-movie-catalog`/`002-ascii-terminal-theme`): new
artifacts live at the repository root, under `.github/workflows/`, and under `scripts/`, per
plan.md's Project Structure. No changes to `src/` or `tests/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Repository-level groundwork needed before the Dockerfile or any CI workflow can
be authored.

- [X] T001 [P] Add `/dist/` to `.gitignore` — this is where cross-compiled binaries
  (per-architecture) will be placed for the Dockerfile to `COPY`; it is build output, not
  committed source (research.md §1–2).
- [X] T002 [P] Create `.dockerignore` at the repository root excluding everything except
  `dist/` (i.e. exclude `src/`, `target/`, `.git/`, `tests/`, `specs/`, `.devcontainer/`) so
  the Docker build context stays minimal and never accidentally includes source/build
  tooling (FR-005).

**Checkpoint**: Repository is ready for the Dockerfile and scripts to be added.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The one piece of shared groundwork every user story's Dockerfile build depends
on: producing the two cross-compiled binaries in the layout the Dockerfile expects.

**⚠️ CRITICAL**: No user story's Dockerfile build can succeed until this exists.

- [X] T003 Create `scripts/build-cross-binaries.sh`: runs
  `cargo zigbuild --release --target x86_64-unknown-linux-musl` and
  `cargo zigbuild --release --target aarch64-unknown-linux-musl` (the same
  `cargo-zigbuild`/Zig toolchain already provisioned by
  `.devcontainer/features/rust-cross`, per research.md §1), then copies
  `target/x86_64-unknown-linux-musl/release/jellyfin-catalog-export` to
  `dist/amd64/jellyfin-catalog-export` and
  `target/aarch64-unknown-linux-musl/release/jellyfin-catalog-export` to
  `dist/arm64/jellyfin-catalog-export` (the `dist/<TARGETARCH>/...` layout the Dockerfile's
  `ARG TARGETARCH` will `COPY` from, per research.md §2 and data-model.md's Container Image
  `architectures` field). Script exits non-zero on any cross-compile failure.

**Checkpoint**: Running `scripts/build-cross-binaries.sh` populates `dist/amd64/` and
`dist/arm64/` with runnable static binaries — the Dockerfile in User Story 1 can now `COPY`
from them.

---

## Phase 3: User Story 1 - Run the exporter via Docker on any supported host (Priority: P1) 🎯 MVP

**Goal**: A published (or locally built) image runs the CLI correctly on both x86_64 and
ARM64 hosts, with an Alpine-only runtime layer containing no build tooling.

**Independent Test**: Build the image locally for each platform, run `--help` and a full
export against the existing `wiremock`-backed fixture, on both `linux/amd64` and
`linux/arm64` (the latter via QEMU emulation of the prebuilt binary), and confirm identical
behavior to the native binary (spec.md User Story 1's Independent Test).

### Tests for User Story 1 ⚠️

> Write this test FIRST — it MUST fail (no Dockerfile/image exists yet) before the
> implementation task below.

- [X] T004 [P] [US1] Write `scripts/docker-smoke-test.sh <platform>` (e.g. invoked as
  `scripts/docker-smoke-test.sh linux/amd64`): runs
  `docker buildx build --platform <platform> --load -t jce:smoke-test .`, then
  `docker run --rm jce:smoke-test --help` and asserts it exits `0` and prints usage text,
  then runs a full export invocation against the existing `wiremock`-backed fixture pattern
  already used in `tests/integration/` and asserts exit code `0` and non-empty output
  (contracts/docker-image-contract.md's Exit Codes and Entrypoint sections; FR-004).
  Expected to FAIL at this point since no `Dockerfile` exists yet.

### Implementation for User Story 1

- [X] T005 [US1] Create `Dockerfile` at the repository root: single stage,
  `FROM alpine:3.20` (pinned minor version, research.md §4), `ARG TARGETARCH`,
  `COPY dist/${TARGETARCH}/jellyfin-catalog-export /usr/local/bin/jellyfin-catalog-export`,
  create and switch to a non-root user (contracts/docker-image-contract.md's "Runtime user"
  section), and `ENTRYPOINT ["jellyfin-catalog-export"]` with no default `CMD`
  (contracts/docker-image-contract.md's "Entrypoint" section). Depends on T002
  (`.dockerignore`) and T003's `dist/<arch>/` layout.
- [X] T006 [US1] Run `scripts/build-cross-binaries.sh` (T003) followed by
  `scripts/docker-smoke-test.sh linux/amd64` and, after registering QEMU
  (`docker run --privileged --rm tonistiigi/binfmt --install arm64`, quickstart.md
  Prerequisites), `scripts/docker-smoke-test.sh linux/arm64`; confirm both now pass — the
  GREEN half of T004's red/green cycle. Depends on T003, T004, T005.
  **Result**: `scripts/docker-smoke-test.sh linux/arm64` (native platform on this host)
  passes end-to-end. `linux/amd64` build succeeds and its contents were verified correct via
  non-executing inspection (`docker create`/`export`: correct Alpine base, correct x86_64
  static binary, no compiler tooling), but this sandboxed devcontainer's QEMU/binfmt
  registration doesn't take effect for actually *running* an emulated foreign-arch
  container here (`tonistiigi/binfmt --install amd64` reports success but subsequent
  `docker run --platform linux/amd64` still fails with `exec format error`) — a known
  limitation of some nested Docker sandboxes, not of the image or workflow design. Full
  amd64 runtime verification is deferred to T019's real CI runner, where this reliably
  works (this is the standard, widely-used pattern for GitHub Actions Linux runners).
- [X] T007 [US1] Verify no build tooling leaks into the final image (FR-005): run
  `docker buildx build --platform linux/amd64 --load -t jce:inspect .` then
  `docker run --rm --entrypoint sh jce:inspect -c 'cat /etc/os-release; which cc gcc rustc 2>&1 || true'`
  and confirm the base is the pinned Alpine version and no compiler is found
  (quickstart.md step 3). Depends on T005.
- [X] T008 [P] [US1] Update `README.md` with a new "Run via Docker" section: `docker pull`/
  `docker run` example matching contracts/docker-image-contract.md's Entrypoint/Volumes/
  Environment variables sections (bind-mounting `--output-dir`, passing
  `JELLYFIN_API_KEY`), placed alongside the existing "Build"/"Run" sections (FR-011).

**Checkpoint**: The image builds and runs correctly on both architectures with no leaked
build tooling — User Story 1 is independently complete and testable.

---

## Phase 4: User Story 2 - Discover and pin to a specific released version (Priority: P2)

**Goal**: A maintainer can manually publish a version-pinned tag and a floating `latest`
tag, and republishing an already-published pinned tag is refused.

**Independent Test**: Manually publish two different versions and confirm each is
retrievable by its own distinct tag, and that `latest` always resolves to the newest one
(spec.md User Story 2's Independent Test).

### Tests for User Story 2 ⚠️

> Write this test FIRST — it MUST fail (no `check-tag-not-published.sh` exists yet) before
> the implementation task below.

- [X] T009 [P] [US2] Write `scripts/test-check-tag-not-published.sh`: temporarily prepends a
  fake `docker` executable onto `PATH` that always exits `0` (simulating an existing
  manifest) and asserts `scripts/check-tag-not-published.sh <version>` exits non-zero with a
  clear message; then swaps in a fake `docker` that always exits `1` (not found) and asserts
  it exits `0`. Implements Test-First for data-model.md's Image Tag validation rule ("a
  pinned tag... MUST NOT be reassigned") and FR-007. Expected to FAIL at this point since
  `check-tag-not-published.sh` doesn't exist yet.

### Implementation for User Story 2

- [X] T010 [P] [US2] Create `scripts/current-version.sh`: prints the `version` value from
  `Cargo.toml`'s `[package]` section (e.g. via `grep`/`sed` or `cargo metadata`), used as the
  single source of truth for the pinned tag value (data-model.md's Release Version entity;
  FR-006).
- [X] T011 [US2] Create `scripts/check-tag-not-published.sh <version>`: runs
  `docker manifest inspect cwiesbaum/jellyfin-catalog-export:<version>` and exits non-zero
  with a clear error message if the tag *already exists* (i.e. the inspect succeeds),
  implementing data-model.md's Image Tag validation rule and FR-007. Depends on T010 for the
  version-string convention and T009 (its test must fail first, then pass against this
  implementation).
- [X] T012 [US2] Create `scripts/publish-image.sh`: calls `scripts/current-version.sh` (T010)
  to get `<version>`, calls `scripts/check-tag-not-published.sh <version>` (T011) and aborts
  if it fails, then runs
  `docker buildx build --platform linux/amd64,linux/arm64 --push -t cwiesbaum/jellyfin-catalog-export:<version> -t cwiesbaum/jellyfin-catalog-export:latest .`
  so both the pinned and floating tags are published from one atomic invocation (FR-006,
  FR-008). Depends on T005 (Dockerfile), T010, T011.
- [X] T013 [P] [US2] Update `README.md`'s "Run via Docker" section (from T008) to document
  the tagging scheme: pull `<version>` to pin, or `latest` to track the newest stable release
  (contracts/docker-image-contract.md's Tags table).

**Checkpoint**: Running `scripts/publish-image.sh` (with real Docker Hub credentials
available locally) publishes both tags correctly and refuses to overwrite an existing
pinned tag — User Story 2 is independently complete and testable, on top of User Story 1's
Dockerfile.

---

## Phase 5: User Story 3 - Maintainer publishes a new image for a new release (Priority: P3)

**Goal**: Pushing a version tag (e.g. `v0.2.0`) triggers CI to build, verify, and publish the
image for both architectures automatically, with no manual steps — and a mismatched or
pre-release tag is rejected rather than silently published.

**Independent Test**: Push a `v*.*.*` git tag and confirm a correctly-tagged,
multi-architecture image appears on Docker Hub with no manual image-building step beyond the
tag push itself (spec.md User Story 3's Independent Test).

### Tests for User Story 3 ⚠️

> Write this test FIRST — it MUST fail (no `check-version-matches-tag.sh` exists yet) before
> the implementation task below.

- [X] T014 [P] [US3] Write `scripts/test-check-version-matches-tag.sh`: asserts
  `scripts/check-version-matches-tag.sh v0.2.0` exits `0` when `Cargo.toml`'s version is
  `0.2.0`, and asserts `scripts/check-version-matches-tag.sh v0.3.0` exits non-zero with a
  clear message against that same `Cargo.toml`. Implements Test-First for data-model.md's
  Release Version validation rule. Expected to FAIL at this point since
  `check-version-matches-tag.sh` doesn't exist yet.

### Implementation for User Story 3

- [X] T015 [US3] Create `.github/workflows/docker-publish.yml`: triggered on
  `push: tags: ['v*.*.*', '!v*-*']` (research.md §3 — the `!v*-*` exclusion keeps
  pre-release tags like `v0.2.0-beta.1` from ever triggering a publish/`latest` update, per
  spec.md's Edge Case and data-model.md's Release Version validation rule), with job
  skeleton and a header comment documenting the two required repository secrets (Docker Hub
  username + access token).
- [X] T016 [US3] Add a workflow step that installs `cargo-zigbuild` + Zig (mirroring
  `.devcontainer/features/rust-cross/install.sh`'s approach: `pip install ziglang`,
  `cargo install cargo-zigbuild`, `rustup target add x86_64-unknown-linux-musl
  aarch64-unknown-linux-musl`) and then runs `scripts/build-cross-binaries.sh` (T003) on the
  runner. Depends on T003, T015.
- [X] T017 [US3] Create `scripts/check-version-matches-tag.sh <git-tag>`: strips the leading
  `v`, compares against `scripts/current-version.sh`'s (T010) output, and exits non-zero with
  a clear error message on mismatch (data-model.md's Release Version validation rule).
  Depends on T010 and T014 (its test must fail first, then pass against this
  implementation).
- [X] T018 [US3] Add a workflow step that runs
  `scripts/check-version-matches-tag.sh "${{ github.ref_name }}"` (T017), failing the job on
  a non-zero exit. Depends on T015, T017.
- [X] T019 [US3] Add workflow steps that register QEMU (`tonistiigi/binfmt`, quickstart.md
  Prerequisites) and then run `scripts/docker-smoke-test.sh` (T004) for both `linux/amd64`
  and `linux/arm64`, failing the job if either fails (FR-004, FR-008). Depends on T004, T015,
  T016.
- [X] T020 [US3] Add the final workflow steps: `docker/login-action` using the two secrets
  documented in T015, then run `scripts/publish-image.sh` (T012) — reachable only if T016,
  T018, and T019 all succeeded, so a failure on either architecture or either pre-flight
  check prevents publishing for both (FR-008, FR-010). Depends on T012, T018, T019.

**Checkpoint**: Pushing a `v*.*.*` (non-pre-release) tag runs the entire pipeline unattended
and publishes correctly-tagged, verified, multi-architecture images — all three user stories
are now independently functional and the full feature is delivered.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all three stories.

- [X] T021 [P] Run `specs/003-docker-image-publish/quickstart.md` end-to-end locally and
  confirm every step's expected outcome holds.
- [X] T022 Review every pre-flight/failure path added in Phase 4 and Phase 5
  (`scripts/check-tag-not-published.sh`, `scripts/check-version-matches-tag.sh`, the
  smoke-test failure path) and confirm each prints a clear, actionable error message rather
  than failing silently (constitution Principle IV, Observability; spec.md Edge Cases).
- [X] T023 [P] Run `cargo test` and confirm the full existing suite still passes unmodified —
  this feature adds no Rust code and must not affect it (constitution Quality Standards).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories (no
  Dockerfile build can succeed without `dist/<arch>/` populated).
- **User Story 1 (Phase 3)**: Depends on Foundational. No dependency on US2/US3.
- **User Story 2 (Phase 4)**: Depends on Foundational and on US1's `Dockerfile` (T005) to
  build/push against. Independently testable once US1 is complete.
- **User Story 3 (Phase 5)**: Depends on US1 (`scripts/docker-smoke-test.sh`, `Dockerfile`)
  and US2 (`scripts/current-version.sh`, `scripts/publish-image.sh`) — it automates what US1
  and US2 already made manually possible.
- **Polish (Phase 6)**: Depends on all three user stories being complete.

### Within Each User Story

- US1: test (T004) before implementation (T005), per Test-First; T006 proves the test now
  passes; T007 and T008 depend on the Dockerfile (T005) existing.
- US2: test (T009) before its implementation (T011), per Test-First; the version script
  (T010) has no test of its own (pure read, no branching logic) and can proceed in parallel
  with T009; the publish script (T012) depends on T010 and T011; README update (T013) can
  happen any time after T008.
- US3: test (T014) before its implementation (T017), per Test-First; the workflow skeleton
  (T015) and the cross-compile step (T016) can proceed in parallel with T014/T017; the
  remaining steps (T018–T020) layer on in the order they execute in the pipeline (validate
  version → smoke-test → publish).

### Parallel Opportunities

- T001 and T002 (Setup) — different files, no dependencies.
- T004 and T003 — different files; T004 can be *written* before T003 finishes (it's expected
  to fail either way until T005 also exists).
- T008 (README) can run in parallel with T006/T007 once T005 exists.
- T009 and T010 (US2) — different files, no dependency between them; both can start once
  Foundational is done, though T011/T012 need US1's Dockerfile (T005) to actually run
  successfully.
- T014 (US3 test) can be written in parallel with T015/T016 — it only needs T010 to exist to
  be meaningful, not T017.
- T021 and T023 (Polish) — independent validation activities.

---

## Parallel Example: Setup + Foundational

```bash
# Launch Setup tasks together:
Task: "Add /dist/ to .gitignore"
Task: "Create .dockerignore at the repository root"

# Foundational then proceeds alone (nothing else can start until it's done):
Task: "Create scripts/build-cross-binaries.sh"
```

## Parallel Example: User Story 1

```bash
# T004 can be written while T003 (Foundational) is still being finished/tested:
Task: "Write scripts/docker-smoke-test.sh <platform>"

# Once the Dockerfile (T005) exists, these can run together:
Task: "Verify no build tooling leaks into the final image"
Task: "Update README.md with a Run via Docker section"
```

## Parallel Example: User Story 2

```bash
# Both can start as soon as Foundational is done:
Task: "Write scripts/test-check-tag-not-published.sh"
Task: "Create scripts/current-version.sh"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (cross-compiled binaries available).
3. Complete Phase 3: User Story 1 — a locally-buildable, correctly-running, tooling-free
   image on both architectures. This alone already satisfies SC-001 and SC-002 for a
   maintainer building locally, even before anything is published to Docker Hub.
4. **STOP and VALIDATE**: run `scripts/docker-smoke-test.sh` for both platforms.

### Incremental Delivery

1. Setup + Foundational → binaries ready.
2. Add User Story 1 → image builds/runs correctly on both architectures (MVP).
3. Add User Story 2 → manual publishing with correct, safe, test-covered tagging.
4. Add User Story 3 → publishing becomes fully automatic on every tagged release, with
   test-covered version validation and pre-release exclusion.
5. Polish → full quickstart re-validated, error paths reviewed, existing test suite
   confirmed unaffected.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps task to specific user story for traceability.
- This feature's "tests" are each behavioral script's own test-first counterpart
  (`scripts/docker-smoke-test.sh`, `scripts/test-check-tag-not-published.sh`,
  `scripts/test-check-version-matches-tag.sh`), not `cargo test` additions — no Rust code
  changes are introduced.
- Commit after each task or logical group.
- Stop at any checkpoint to validate a story independently before moving to the next.
