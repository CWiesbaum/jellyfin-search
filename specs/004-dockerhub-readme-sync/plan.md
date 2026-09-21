# Implementation Plan: Docker Hub Repository Overview Sync

**Branch**: `004-dockerhub-readme-sync` | **Date**: 2026-09-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-dockerhub-readme-sync/spec.md`

## Summary

Extend the existing `.github/workflows/docker-publish.yml` (from `003-docker-image-publish`)
with a second job that sets the Docker Hub repository's short description and full overview
(the repository's README) every time a release is published, using the well-established
`peter-evans/dockerhub-description` GitHub Action rather than hand-rolling Docker Hub Hub
API calls. The README gains a link back to the GitHub repository so that the overview page
— sourced verbatim from the README — gives Docker Hub visitors a way back to the source.
The sync job depends on the image-publish job succeeding on a real release, but is also
independently runnable via a `workflow_dispatch` trigger (e.g. after a README-only edit),
without needing to cut a new release — see "Independent trigger" below. Being a separate
job, a metadata-sync failure is visible but can never undo or invalidate an
already-published image tag.

### Independent trigger (post-implementation refinement)

The initial implementation placed the sync as the last *step* of the same job as the image
build/publish, which meant it could only ever run as a side effect of a full release — there
was no way to push a README-only correction to Docker Hub without bumping the version. This
was refined into two jobs in the same workflow:

- `publish` — unchanged, gated by `if: github.event_name == 'push'` (only runs for real tag
  pushes).
- `sync-dockerhub-overview` — `needs: publish`, with
  `if: always() && (needs.publish.result == 'success' || needs.publish.result == 'skipped')`.
  On a tag push, this waits for `publish` to succeed (same guarantee as before). On a manual
  `workflow_dispatch` run, `publish` is skipped by its own `if:`, and `skipped` still
  satisfies this job's condition, so it runs standalone.

The workflow's `on:` block gained a `workflow_dispatch: {}` trigger to make this possible
from the Actions UI/API on demand.

## Technical Context

**Language/Version**: N/A change — this feature touches only CI workflow YAML and
documentation (README.md); no Rust code is added or modified.

**Primary Dependencies**: `peter-evans/dockerhub-description@v4` (a widely-used, maintained
GitHub Action that wraps Docker Hub's Hub API v2 login + repository-update calls — chosen
over a hand-rolled `curl`/API-token script for the same reason `003-docker-image-publish`
used `docker/login-action` and `docker/setup-buildx-action` rather than reimplementing their
logic: Simplicity/YAGNI). Reuses the existing `DOCKERHUB_USERNAME` secret, but requires a
**new** `DOCKERHUB_DESCRIPTION_TOKEN` secret rather than reusing `DOCKERHUB_TOKEN`
(discovered when the first real run failed with "Error: Forbidden"): the action's Hub API
call to update repository metadata requires a token with **Read/Write/Delete** scope, while
`DOCKERHUB_TOKEN` is scoped Read/Write only (sufficient for `docker/login-action`'s image
push, per the action's own documented requirement). Kept as a separate secret rather than
widening `DOCKERHUB_TOKEN`'s scope, so the image-push job doesn't run with delete-capable
credentials it doesn't need.

**Storage**: N/A.

**Testing**: `scripts/check-readme-has-github-link.sh` asserts `README.md` contains the
GitHub repository URL — written first, expected to fail (no link yet), then made to pass
once the link is added (Test-First, applied to this feature's actual, verifiable
deliverable). The Hub API sync step itself is not unit-testable locally (it requires live
Docker Hub credentials and mutates real, rate-limited external state); it's validated by
`quickstart.md`'s read-only check against Docker Hub's public repository API before/after a
real publish, matching `003-docker-image-publish`'s precedent of deferring true end-to-end
proof of CI-only behavior to a real workflow run.

**Target Platform**: Same as `003-docker-image-publish` — GitHub Actions Linux runner,
Docker Hub.

**Project Type**: Single project (unchanged) — this feature adds one script and one
workflow step to the existing packaging/CI surface from `003-docker-image-publish`, plus a
README edit. No `src/` or `tests/` changes.

**Performance Goals**: N/A.

**Constraints**:
- The short description MUST fit Docker Hub's documented 100-character limit for that
  field.
- A failure in the description/overview sync job MUST NOT cause the already-published
  image tag(s) from a same-run `publish` job to be treated as invalid, removed, or rolled
  back (FR-007) — achieved structurally by making the sync a separate job that only starts
  after `publish` has already succeeded (or was skipped, for a manual run); one job's
  failure has no effect on another job's already-reported result in GitHub Actions.
- The sync job's own failure MUST surface clearly in the workflow run (FR-006) — satisfied
  by not suppressing the action's exit status (no `continue-on-error: true`, no `|| true`).

**Scale/Scope**: One new script (`scripts/check-readme-has-github-link.sh`), one new job
(`sync-dockerhub-overview`) plus a `workflow_dispatch` trigger added to the existing
`.github/workflows/docker-publish.yml`, and a small README edit (one link line). No new
application modules, no new registries beyond Docker Hub.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Test-First (NON-NEGOTIABLE)** — PASS. `scripts/check-readme-has-github-link.sh` is
  written and run before the README link exists (expected to fail), then made to pass once
  the link is added.
- **II. Simplicity & YAGNI** — PASS. One third-party action reused instead of hand-rolled
  Hub API calls; one small script; a one-line README addition; no new abstractions,
  configuration surface, or registries.
- **III. Code Review Discipline** — PASS (process-level; no plan-time design impact).
- **IV. Observability** — PASS: the sync step's failure is not suppressed, so it fails
  loudly in the Actions run log (spec FR-006); the README-link check script prints a clear
  message identifying what's missing.
- **V. Semantic Versioning & Breaking Changes** — PASS/N/A. This feature changes no public
  CLI, image, or data contract — it only affects Docker Hub's own metadata display and the
  README's prose, so no contract version implications arise.

No violations identified. Complexity Tracking table is not needed.

**Post-Design Re-check** (after Phase 1): The data model (Docker Hub Repository Metadata,
below) introduces no new dependencies, services, or abstractions beyond what is listed
above. All gates remain PASS.

## Project Structure

### Documentation (this feature)

```text
specs/004-dockerhub-readme-sync/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
└── quickstart.md        # Phase 1 output (/speckit-plan command)
```

No `contracts/` directory is produced for this feature: it introduces no new CLI flag, no
new exit code, and no new/changed image interface — `003-docker-image-publish`'s
`contracts/docker-image-contract.md` remains complete and unaffected. This mirrors
`002-ascii-terminal-theme`, which also skipped `contracts/` for the same reason.

### Source Code (repository root)

```text
README.md                           # MODIFIED: add a link to the GitHub repository
                                     # (FR-004), near the top so it's visible whether the
                                     # page is rendered on GitHub or on Docker Hub.

scripts/
└── check-readme-has-github-link.sh # NEW: asserts README.md contains the GitHub
                                     # repository URL; exits non-zero with a clear message
                                     # if it's missing.

.github/
└── workflows/
    └── docker-publish.yml          # MODIFIED: adds a `workflow_dispatch` trigger and a
                                     # new `sync-dockerhub-overview` job (needs: publish;
                                     # runs when publish succeeds OR was skipped, so it also
                                     # runs standalone on manual dispatch), using
                                     # peter-evans/dockerhub-description@v4 with the
                                     # existing DOCKERHUB_USERNAME secret, a new
                                     # DOCKERHUB_DESCRIPTION_TOKEN secret (Read/Write/Delete
                                     # scope — see Primary Dependencies above), a literal
                                     # short-description string, and readme-filepath
                                     # pointing at ./README.md.
```

Every existing file under `src/` and `tests/` is unchanged by this feature — it is purely a
CI-workflow and documentation addition, consistent with how `002-ascii-terminal-theme` and
`003-docker-image-publish` each touched only the files their own scope required.

**Structure Decision**: Option 1 (single project), unchanged. This feature extends the
existing CI/packaging surface from `003-docker-image-publish` rather than introducing a new
component.

## Complexity Tracking

*No Constitution Check violations were identified — this table is intentionally empty.*
