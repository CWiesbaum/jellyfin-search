# Quickstart: Validating the Docker Hub Overview Sync

This walks through validating the feature both locally (what's checkable without live
Docker Hub credentials) and against the real Docker Hub repository once a release is
published. See `data-model.md` for the fields being synced.

## 1. Validate the README link (local, no credentials needed)

```bash
./scripts/check-readme-has-github-link.sh
```

**Expected outcome**: exits `0` and confirms `README.md` contains
`https://github.com/CWiesbaum/jellyfin-search`. Before the link is added, this is expected
to fail — that failure is the "red" half of this feature's test-first cycle (plan.md's
Testing section).

## 2. Check Docker Hub's current state (read-only, no credentials needed)

Docker Hub's repository-info endpoint is public read access:

```bash
curl -s https://hub.docker.com/v2/repositories/cwiesbaum/jellyfin-catalog-export/ \
  | grep -o '"description":"[^"]*"'
```

**Expected outcome** (before this feature's workflow step has ever run): `"description":""`
— confirmed empty as of 2026-09-21, i.e. the baseline this feature changes. This public
endpoint doesn't expose `full_description` at all (verified: the field is simply absent
from its response); confirm that field visually instead — see step 4.

## 3. Inspect the new workflow step (local, no credentials needed)

```bash
grep -A6 'dockerhub-description' .github/workflows/docker-publish.yml
```

**Expected outcome**: confirms the step uses `peter-evans/dockerhub-description@v4`, the
existing `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` secrets, `readme-filepath: ./README.md`, and
a `short-description` under 100 characters (research.md §1, §3) — and that it comes *after*
the "Publish multi-architecture image" step (research.md §2).

## 4. Full validation (real publish, or a manual sync)

Either push a `v*.*.*` git tag as usual to trigger the full `publish` job followed by
`sync-dockerhub-overview`, **or** trigger `sync-dockerhub-overview` on its own — via
GitHub's Actions UI ("Run workflow" on `docker-publish.yml`) or
`gh workflow run docker-publish.yml` — to push the current README/description without
cutting a release (e.g. after a README-only fix). Once the run completes successfully,
repeat step 2's `curl` check:

**Expected outcome**: `"description"` now matches the short description string from the
workflow (no longer `""`). Confirm the full overview visually by opening
`https://hub.docker.com/r/cwiesbaum/jellyfin-catalog-export` and checking that it renders
`README.md`'s content with a working link back to
`https://github.com/CWiesbaum/jellyfin-search` (spec User Stories 1 and 2's Independent
Tests).
