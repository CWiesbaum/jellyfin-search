# Phase 0 Research: Docker Hub Repository Overview Sync

## 1. How to update Docker Hub's repository description and overview from CI

**Decision**: Use the `peter-evans/dockerhub-description@v4` GitHub Action as a new step in
the existing `.github/workflows/docker-publish.yml`, authenticating with the same
`DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` repository secrets already used by
`docker/login-action`, with `readme-filepath: ./README.md` and a literal `short-description`
string.

**Rationale**: Docker Hub has no first-class `docker` CLI or `buildx` command for updating a
repository's short/full description — only its Hub API v2 (`POST /v2/users/login/` then
`PATCH /v2/repositories/<namespace>/<repo>/`). `peter-evans/dockerhub-description` is a
widely-used, actively maintained action that wraps exactly that call, matching this
project's existing precedent (`003-docker-image-publish` used `docker/login-action` and
`docker/setup-buildx-action` rather than reimplementing their logic) and the constitution's
Simplicity/YAGNI principle: hand-rolling a `curl`-based Hub API script would duplicate
well-tested, already-solved logic for no benefit.

**Alternatives considered**:
- Hand-rolled `curl` script calling the Hub API v2 login + patch endpoints directly —
  rejected: more code to maintain and test for a solved problem, with no added flexibility
  this feature actually needs.
- `docker/build-push-action`'s own annotations/labels (OCI image labels like
  `org.opencontainers.image.description`) — rejected: those annotate the *image manifest*,
  not the Docker Hub repository page's own description/overview fields, which is what the
  spec asks for (FR-001/FR-002 are explicitly about the repository page, not image
  metadata).

## 2. Where the new step belongs in the existing workflow

**Decision**: Append the description-sync step to the end of the `publish` job, after the
existing "Publish multi-architecture image" step.

**Rationale**: Running it last means the image publish (the higher-stakes, harder-to-redo
operation) always completes first; a metadata-sync failure then fails the job visibly
(FR-006) without threatening or rolling back the already-published image tags (FR-007),
since GitHub Actions doesn't undo a completed step when a later one in the same job fails.

**Alternatives considered**:
- Running it before the image publish, or as a separate parallel job — rejected: gives no
  benefit (the sync doesn't need to happen before the image exists) and would only add
  complexity (a second job needs its own checkout/secrets wiring) for no functional gain.

## 3. Short description content

**Decision**: A literal, hand-authored one-line string reusing this project's own existing
CLI `about` text ("Export a Jellyfin movie library to a self-contained static site" — 65
characters, well under Docker Hub's 100-character limit for this field), passed directly as
the action's `short-description` input.

**Rationale**: A reasonable default per spec's Assumptions — the exact wording is a
copywriting detail, not something requiring a derived/templated source of truth. Reusing
already-established, user-facing project language keeps it consistent with `--help` output
users may already have seen.

**Alternatives considered**:
- Deriving it programmatically from `Cargo.toml` (which has no `description` field) or from
  the README's first sentence — rejected: adds a parsing step for a single short string that
  changes rarely, with no requirement calling for automatic derivation (Simplicity/YAGNI).

## 4. Verifying the README-to-GitHub link exists

**Decision**: `scripts/check-readme-has-github-link.sh` greps `README.md` for the literal
GitHub repository URL (`https://github.com/CWiesbaum/jellyfin-search`) and exits non-zero
with a clear message if it's absent.

**Rationale**: This is the one piece of this feature's actual deliverable that's cheaply,
locally testable without live credentials or external API calls — applying Test-First to it
mirrors `003-docker-image-publish`'s pattern of small, purpose-built bash checks
(`check-tag-not-published.sh`, `check-version-matches-tag.sh`) rather than reaching for a
Rust test for something with no Rust-code involvement.

**Alternatives considered**:
- A `cargo test` assertion reading `README.md` — rejected: README content isn't loaded by
  any Rust code path in this project, so a Rust test would only add an indirect, awkward way
  to check a plain-text file; a direct shell check is simpler and matches the file's own
  nature.
