# Implementation Plan: ASCII Terminal Theme

**Branch**: `002-ascii-terminal-theme` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-ascii-terminal-theme/spec.md`

## Summary

Restyle the existing generated movie-overview page (from the `001-remote-movie-catalog`
feature) to look like a classic black-background, green-text computer terminal, with a
hand-authored ASCII-art banner as the page's decorative centerpiece. This is a
presentation-only change confined to the single embedded template asset
(`src/site/assets/index.html.tmpl`): the light/dark auto-switching theme is replaced with
a fixed terminal palette, a monospace font is applied throughout, and real movie
thumbnails are kept (not converted to character-art) but framed with terminal-styled
decoration. No Rust code, CLI flags, or the embedded JSON data schema change — every
existing behavior (search, empty states, placeholders, atomic writes, exit codes) is
carried forward unmodified from `001-remote-movie-catalog`.

## Technical Context

**Language/Version**: Rust (stable toolchain, 2024 edition) — unchanged from
`001-remote-movie-catalog`; this feature does not add or change any Rust logic.

**Primary Dependencies**: None added. The change is entirely CSS/HTML/vanilla-JS within
the single existing template asset, rendered by the unchanged
`src/site/template.rs::render` (plain string substitution, per that feature's research.md
§6). No web fonts or other external assets are introduced — see Constraints below.

**Storage**: N/A (unchanged).

**Testing**: `cargo test`, extending the existing `tests/unit/test_template.rs` with
assertions on the rendered HTML's presentation (fixed dark palette present, no
alternate-theme media query, monospace font declared, ASCII banner present). Critically,
the full pre-existing test suite from `001-remote-movie-catalog` (34 tests: contract,
integration, unit) MUST continue to pass unmodified — it is the regression gate for this
spec's FR-007 ("all existing page functionality... MUST continue to work exactly as
before").

**Target Platform**: Same as `001-remote-movie-catalog` — any modern browser, served as
static files; no server-side rendering.

**Project Type**: Single project (unchanged). This feature touches one file inside the
existing Rust CLI's embedded assets; it does not add a new component.

**Performance Goals**: No change from `001-remote-movie-catalog`. A pure CSS/markup
restyle of a page that already renders a few thousand records instantly has no measurable
performance impact.

**Constraints**:
- MUST NOT introduce any runtime network request (no web fonts, no external stylesheets,
  no `fetch`/`XMLHttpRequest`) — carries forward `001-remote-movie-catalog`'s FR-006 and
  its existing regression test (`generated_page_contains_no_runtime_network_calls`), which
  this feature's tests must continue to satisfy.
- MUST NOT change the embedded catalog JSON schema, the CLI contract, or any Rust module
  outside `src/site/assets/index.html.tmpl` — this is presentation-only, per spec.md's
  Assumptions.
- MUST remain responsive: the movie grid still reflows via CSS Grid
  (`repeat(auto-fill, minmax(140px, 1fr))`) at any viewport width. Hand-authored ASCII art
  has a fixed character grid and cannot reflow, so it is confined to static, non-repeating
  elements (the page banner) rather than to per-movie framing — see research.md §1.
- Per FR-003, movie titles and the search input MUST remain plain, unstyled-as-art text —
  ASCII decoration must not touch actual content, only chrome around it.

**Scale/Scope**: Same as `001-remote-movie-catalog` (up to several thousand movies); no
change in scope introduced by this feature.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Test-First (NON-NEGOTIABLE)** — PASS. New presentation assertions
  (`tests/unit/test_template.rs`) will be written and observed to fail before the
  template is restyled; the full existing suite acts as a regression gate written well
  before this feature existed.
- **II. Simplicity & YAGNI** — PASS. No new dependencies, no new files, no new Rust
  modules. Decorative ASCII art is plain static text baked into the one existing
  template, not a generated-at-runtime capability (research.md §1 explains why literal
  ASCII art is deliberately *not* applied to dynamic per-movie content).
- **III. Code Review Discipline** — PASS (process-level; no plan-time design impact).
- **IV. Observability** — PASS (no change: no new failure modes, no new logging surface;
  existing exit codes and stderr/stdout contracts are untouched).
- **V. Semantic Versioning & Breaking Changes** — PASS. Neither `contracts/cli-interface.md`
  nor `contracts/movie-data-schema.md` from `001-remote-movie-catalog` change — this
  feature adds no new public interface, so no contract version implications arise from it.

No violations identified. Complexity Tracking table is not needed.

**Post-Design Re-check** (after Phase 1): The data-model (presentation tokens only) and
quickstart introduce no new dependencies, services, or abstractions beyond what is listed
above. All gates remain PASS.

## Project Structure

### Documentation (this feature)

```text
specs/002-ascii-terminal-theme/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md         # Phase 1 output (/speckit-plan command) — presentation tokens, no new data entities
├── quickstart.md         # Phase 1 output (/speckit-plan command)
└── tasks.md              # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

No `contracts/` directory is produced for this feature: it introduces no new CLI flag, no
new exit code, and no new/changed embedded JSON field — the two contracts already
published under `specs/001-remote-movie-catalog/contracts/` remain complete and
unaffected, so there is nothing new to contract here.

### Source Code (repository root)

```text
src/
└── site/
    └── assets/
        └── index.html.tmpl   # MODIFIED IN PLACE: CSS palette/typography + ASCII banner
                                # markup + minor JS-rendered class/attribute adjustments
                                # for the terminal-framed thumbnail treatment.

tests/
└── unit/
    └── test_template.rs      # EXTENDED: new assertions on rendered presentation
```

Every other file in the repository (all of `src/cli.rs`, `src/jellyfin/`, `src/catalog/`,
`src/site/generator.rs`, `src/site/template.rs`, `src/lib.rs`, `src/main.rs`, and the rest
of `tests/`) is unchanged by this feature.

**Structure Decision**: Option 1 (single project), unchanged from
`001-remote-movie-catalog`. This feature is a targeted modification of one asset inside
that existing structure, not a new component.

## Complexity Tracking

*No Constitution Check violations were identified — this table is intentionally empty.*
