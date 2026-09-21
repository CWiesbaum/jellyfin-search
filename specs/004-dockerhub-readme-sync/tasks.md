---

description: "Task list template for feature implementation"
---

# Tasks: Docker Hub Repository Overview Sync

**Input**: Design documents from `/specs/004-dockerhub-readme-sync/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md,
data-model.md

**Tests**: Included. The constitution's Principle I (Test-First, NON-NEGOTIABLE) applies to
this feature's one locally-verifiable deliverable: `scripts/check-readme-has-github-link.sh`
is written before the GitHub link exists in `README.md`, expected to fail, then made to pass
once the link is added.

**Organization**: Tasks are grouped by user story (from spec.md) to enable independent
implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project (unchanged from `001-remote-movie-catalog`/`002-ascii-terminal-theme`/
`003-docker-image-publish`): this feature only touches `.github/workflows/docker-publish.yml`,
`README.md`, and one new file under `scripts/`, per plan.md's Project Structure.

---

## Phase 1: Setup

**Not applicable to this feature** — there is no shared project initialization to do. The
workflow file, README, and `scripts/` directory this feature edits/extends all already
exist from `003-docker-image-publish`.

---

## Phase 2: Foundational

**Not applicable to this feature** — User Story 1's workflow-step edit and User Story 2's
README edit touch different files with no shared prerequisite between them; neither blocks
the other, so there is no separate foundational phase to complete first.

---

## Phase 3: User Story 1 - Understand the image from Docker Hub alone (Priority: P1) 🎯 MVP

**Goal**: The Docker Hub repository page shows a concise short description and a full
overview matching the project's README, populated automatically by the existing publish
workflow.

**Independent Test**: Publish a release and check the Docker Hub repository page directly
(no image pull needed) for a non-empty short description and a full overview containing the
project's usage instructions (spec.md User Story 1's Independent Test).

### Implementation for User Story 1

- [X] T001 [US1] Add a new step to `.github/workflows/docker-publish.yml`, appended after
  the existing "Publish multi-architecture image" step (research.md §2 — so a metadata-sync
  failure can never threaten an already-published image, per FR-007), using
  `peter-evans/dockerhub-description@v4` (research.md §1) with: `username`/`password` from
  the existing `secrets.DOCKERHUB_USERNAME`/`secrets.DOCKERHUB_TOKEN`, `repository:
  cwiesbaum/jellyfin-catalog-export`, `readme-filepath: ./README.md`, and a literal
  `short-description` reusing the CLI's existing "Export a Jellyfin movie library to a
  self-contained static site" text — which data-model.md's validation rule requires be
  **≤ 100 characters** (it is 65; research.md §3). Do not add `continue-on-error` or
  suppress its exit status (constitution Principle IV, Observability; FR-006).
  **Refined post-completion**: moved into its own `sync-dockerhub-overview` job
  (`needs: publish`, running when `publish` succeeds *or* was skipped) plus a
  `workflow_dispatch` trigger on the workflow, so it can also run standalone without a
  release — see plan.md's "Independent trigger" section.
- [X] T002 [US1] Verify the new step's configuration locally per quickstart.md step 3:
  `grep -A6 'dockerhub-description' .github/workflows/docker-publish.yml` and confirm it
  references `peter-evans/dockerhub-description@v4`, both existing secrets,
  `readme-filepath: ./README.md`, a `short-description` under 100 characters, and that it
  appears after the "Publish multi-architecture image" step (not before it). Depends on
  T001.

**Checkpoint**: The next tagged release will populate Docker Hub's short description and
full overview from the current README — User Story 1 is independently complete.

---

## Phase 4: User Story 2 - Reach the source repository from Docker Hub (Priority: P2)

**Goal**: A visitor reading the Docker Hub overview (which is the README's own content, per
User Story 1) can reach the GitHub repository in one click.

**Independent Test**: Open the published Docker Hub repository page and confirm a working
link to the GitHub repository is present and reachable in one click (spec.md User Story 2's
Independent Test).

### Tests for User Story 2 ⚠️

> Write this test FIRST — it MUST fail (no GitHub link in `README.md` yet) before the
> implementation task below.

- [X] T003 [P] [US2] Create `scripts/check-readme-has-github-link.sh`: greps `README.md` for
  the literal URL `https://github.com/CWiesbaum/jellyfin-search` (data-model.md's Docker Hub
  Repository Metadata validation rule: "`full_description` MUST include a link to the GitHub
  repository") and exits non-zero with a clear message if it's absent (research.md §4;
  FR-004). Run it now and confirm it FAILS, since `README.md` doesn't contain that URL yet.

### Implementation for User Story 2

- [X] T004 [US2] Add a link to `https://github.com/CWiesbaum/jellyfin-search` near the top
  of `README.md` (FR-004), placed so it's visible whether the page renders on GitHub or on
  Docker Hub. Depends on T003 (the check must fail first).
- [X] T005 [US2] Run `scripts/check-readme-has-github-link.sh` again and confirm it now
  PASSES — the GREEN half of T003's red/green cycle. Depends on T003, T004.

**Checkpoint**: `README.md` contains a working GitHub link that will be visible in Docker
Hub's overview on the next publish — User Story 2 is independently complete, on top of User
Story 1's workflow step actually pushing that content to Docker Hub.

---

## Phase 5: User Story 3 - Keep Docker Hub in sync without manual upkeep (Priority: P3)

**Goal**: Every future release automatically re-syncs Docker Hub's description and overview
— no separate manual action on Docker Hub's website, ever.

**Independent Test**: Change the README, cut a new release through the existing automated
publish process, and confirm the Docker Hub repository's description and overview reflect
the updated content afterward, without any manual action taken on Docker Hub's website
(spec.md User Story 3's Independent Test).

### Implementation for User Story 3

- [X] T006 [US3] Inspect `.github/workflows/docker-publish.yml` and confirm the step added
  in T001 runs unconditionally on every successful `publish` job execution of the existing
  `push: tags: ['v*.*.*', '!v*-*']` trigger from `003-docker-image-publish` — no
  first-run-only guard, manual `workflow_dispatch` requirement, or other conditional that
  would limit it to a one-time sync (FR-003; spec.md User Story 3's premise). This requires
  no new code: it confirms the same mechanism User Story 1 already added inherently
  satisfies "every release, no manual step" without further changes. Depends on T001.

**Checkpoint**: All three user stories are independently functional — the full feature is
delivered. Every future tagged release keeps Docker Hub's description and overview
automatically current.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all three stories and repo-wide documentation
consistency.

- [X] T007 [P] Run `specs/004-dockerhub-readme-sync/quickstart.md` steps 1–3 end-to-end
  locally (the README-link check, the Docker Hub baseline `curl` check, and the workflow
  step `grep` check) and confirm every step's expected outcome holds.
- [X] T008 Review the new workflow step's failure behavior (constitution Principle IV,
  Observability; FR-006/FR-007): confirm no `continue-on-error: true` or `|| true` anywhere
  suppresses the sync step's exit status, and that its position after "Publish
  multi-architecture image" structurally prevents a sync failure from affecting an
  already-published image tag.
- [X] T009 [P] Update the README's existing intro paragraph (which already links to
  `specs/001-remote-movie-catalog/`, `specs/002-ascii-terminal-theme/`, and
  `specs/003-docker-image-publish/`) to also reference `specs/004-dockerhub-readme-sync/`,
  matching this repository's established convention of linking every shipped feature's spec
  from the README.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup / Foundational**: Not applicable (see Phase 1/2 notes above).
- **User Story 1 (Phase 3)**: No dependencies — can start immediately.
- **User Story 2 (Phase 4)**: No dependency on User Story 1 — it only edits `README.md` and
  a new script, neither of which User Story 1 touches. Independently testable on its own,
  though its *visible effect on Docker Hub* only appears once User Story 1's workflow step
  exists to push the updated README there.
- **User Story 3 (Phase 5)**: Depends on User Story 1 (T001) — it inspects the step User
  Story 1 added.
- **Polish (Phase 6)**: Depends on all three user stories being complete.

### Within Each User Story

- US1: T001 (add the step) before T002 (verify its configuration).
- US2: test (T003) before implementation (T004), per Test-First; T005 proves the test now
  passes.
- US3: T006 depends only on T001 existing; no test-first cycle since it introduces no new
  code, only inspects existing configuration.

### Parallel Opportunities

- T001 (US1, workflow YAML) and T003 (US2, new script) — different files, no dependency
  between them; both can start immediately.
- T007 and T009 (Polish) — independent activities (validation vs. documentation).

---

## Parallel Example: User Story 1 + User Story 2

```bash
# Both can start immediately, in parallel, since they touch different files:
Task: "Add a new step to .github/workflows/docker-publish.yml"
Task: "Create scripts/check-readme-has-github-link.sh"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 3: User Story 1 — the workflow step alone already satisfies SC-001 and
   most of SC-002 (Docker Hub shows a real description and overview) on the very next
   release, even before the GitHub link exists.
2. **STOP and VALIDATE**: run quickstart.md step 3's `grep` check.

### Incremental Delivery

1. Add User Story 1 → Docker Hub gets a real description/overview on the next release
   (MVP).
2. Add User Story 2 → that overview now includes a working link back to GitHub.
3. Add User Story 3 → confirm (no new code needed) that this keeps happening automatically
   on every future release.
4. Polish → quickstart re-validated, failure-handling reviewed, README's spec-links
   convention kept consistent.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps task to specific user story for traceability.
- This feature's only "test" is `scripts/check-readme-has-github-link.sh` — no `cargo test`
  changes are introduced, since no Rust code is added or modified.
- Commit after each task or logical group.
- Stop at any checkpoint to validate a story independently before moving to the next.
