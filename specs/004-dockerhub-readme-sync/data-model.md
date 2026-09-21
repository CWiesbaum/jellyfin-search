# Phase 1 Data Model: Docker Hub Repository Overview Sync

This feature introduces no application data or persistence. The one "entity" below is the
external metadata this feature keeps in sync, not a stored record of this project's own.

## Docker Hub Repository Metadata

The description fields Docker Hub displays on the `cwiesbaum/jellyfin-catalog-export`
repository page.

| Field               | Description                                                          |
|---------------------|------------------------------------------------------------------------|
| `short_description` | A concise, one-line summary of the tool. Docker Hub limits this field to 100 characters (research.md §3). |
| `full_description`  | The repository's rendered overview. Set to the exact content of `README.md` (spec Assumptions). |

**Validation rule**: `short_description` MUST be ≤ 100 characters (Docker Hub's own limit;
research.md §3).

**Validation rule**: `full_description` MUST include a link to the GitHub repository (FR-004)
— enforced locally by `scripts/check-readme-has-github-link.sh` against the source
`README.md` before it's ever pushed to Docker Hub.

**Update trigger**: Both fields are overwritten by
`.github/workflows/docker-publish.yml`'s `sync-dockerhub-overview` job, which runs (a)
automatically after every successful tagged release (same trigger as the image itself, spec
User Story 3), and (b) on demand via that workflow's `workflow_dispatch` trigger, so a
README-only correction can be pushed to Docker Hub without cutting a new release. There is
no update path outside that workflow.
