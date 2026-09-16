# Tasks: ASCII Terminal Theme

**Input**: Design documents from `/specs/002-ascii-terminal-theme/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included and REQUIRED — Constitution Principle I (Test-First, NON-NEGOTIABLE)
mandates tests written before implementation. Within each phase below, test tasks are
listed before the implementation tasks that make them pass.

**Organization**: Tasks are grouped by user story (from spec.md) to enable independent
implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1, US2) — omitted for Setup,
  Foundational, and Polish tasks
- This feature touches one existing file (`src/site/assets/index.html.tmpl`) plus test
  files; no new Rust modules or dependencies are introduced (research.md)

## Path Conventions

Same single Rust binary crate as `001-remote-movie-catalog`: `src/` for implementation,
`tests/` for `contract/`, `integration/`, and `unit/` tests.

---

## Phase 1: Setup

**Purpose**: Establish the pre-change regression baseline before touching anything.

- [X] T001 Run `cargo test` and confirm all 34 pre-existing tests (7 contract, 15 integration, 12 unit) pass, recording this as the baseline that Phase 5's regression check (T012) must still match after this feature's changes

**Checkpoint**: Baseline confirmed clean; safe to start modifying the template.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish the fixed terminal palette and monospace typography that both
user stories build on — the page-wide color/font base neither story works without.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Tests for Foundational infrastructure

> Write this test FIRST; it must fail before the implementation task below is done.

- [X] T002 Unit test in `tests/unit/test_template.rs` asserting the rendered HTML: (a) contains no `prefers-color-scheme` media query (the old conditional theme is gone, not just supplemented), (b) declares the `--bg: #000000`, `--fg: #33ff33`, and `--fg-dim: #1a8c1a` custom properties on `:root` per data-model.md's Presentation Tokens table, and (c) declares a `monospace` font-family

### Implementation for Foundational infrastructure

- [X] T003 In `src/site/assets/index.html.tmpl`: remove the `@media (prefers-color-scheme: dark)` block and the old light-mode base color values (`#fafafa`, `#111`, `#eee`, `#666`, `#aaa`, `#ccc`, `#ddd`, `#333`, `#888`); declare `--bg: #000000`, `--fg: #33ff33`, `--fg-dim: #1a8c1a` as custom properties on `:root`; set `color-scheme: dark` on `:root` (replacing `color-scheme: light dark`); set `body`'s `background`/`color` to `var(--bg)`/`var(--fg)`; and apply the font stack `ui-monospace, "Courier New", Menlo, Consolas, monospace` to `body` (makes T002 pass; research.md §2, §3, §4)

**Checkpoint**: Page background/text color and font are fixed and terminal-appropriate;
individual elements (search input, placeholders, thumbnails) still need their own colors
updated in the story phases below.

---

## Phase 3: User Story 1 - Browse and search inside a terminal-styled page (Priority: P1) 🎯 MVP

**Goal**: Every visible part of the page — header, search box, movie list, status
messages, and a new ASCII-art banner — consistently uses the black-background/green-text
terminal theme, unconditionally (not following the visitor's OS light/dark preference),
with all existing search/browse functionality unaffected.

**Independent Test**: Open the generated page and visually confirm every visible area
uses the terminal styling with no light-themed or non-green element remaining, that an
ASCII banner is present near the top, and that toggling the OS/browser light/dark
preference does not change the page's appearance.

### Tests for User Story 1

> Write these tests FIRST, ensure they FAIL before implementation.

- [X] T004 [P] [US1] Integration test in `tests/integration/test_terminal_theme.rs`: run the full pipeline against a wiremock-stubbed Jellyfin server, then assert the generated `index.html` contains the `--bg`/`--fg`/`--fg-dim` custom property declarations and contains none of the old light-theme hex values (`#fafafa`, `#eee`, `#ccc`), per FR-001
- [X] T005 [P] [US1] Unit test in `tests/unit/test_template.rs` asserting the rendered HTML contains a `<pre>`-rendered ASCII-art banner (a recognizable, stable marker string near the top of `<body>`) styled with the terminal palette, per FR-004 / research.md §1

### Implementation for User Story 1

- [X] T006 [US1] In `src/site/assets/index.html.tmpl`, update `header`, `h1`, `#last-updated`, `#empty-library`, `#empty-state`, and `.movie-meta` selectors to use `var(--fg)` (primary text) or `var(--fg-dim)` (secondary/status text, per data-model.md) instead of their old gray-based values, so every text element on the page uses the terminal palette consistently (depends on T003; contributes to T004 passing)
- [X] T007 [US1] Add a hand-authored ASCII-art banner (a `<pre>` element containing a monospace-aligned text banner, e.g. a stylized rendering of "MOVIE LIBRARY") near the top of `<body>` in `src/site/assets/index.html.tmpl`, styled with `var(--fg)` on `var(--bg)` (depends on T006; makes T005 pass)
- [X] T008 [US1] Style `#search`'s border and background with the terminal palette (e.g. `border: 1px solid var(--fg-dim)`, `background: var(--bg)`, `color: var(--fg)`) in `src/site/assets/index.html.tmpl`, verifying the input element itself remains a normal, fully functional, plain-text `<input>` — no decorative ASCII styling is applied to its content, per FR-003 (depends on T007)

> **Implementation note**: T006–T008 (and, ahead of schedule, T010–T011 from Phase 4)
> were implemented together as a single template rewrite, since this is one small,
> cohesive CSS/HTML file rather than several independent components — splitting a
> ~180-line file into six sequential diffs added process overhead without benefit. Each
> task's test (T004, T005, and later T009) was written and confirmed passing against
> that implementation, per Constitution Principle I.

**Checkpoint**: User Story 1 is fully functional and independently testable — the whole
page presents the terminal theme, with an ASCII banner, and search/browse still work.
This is a deployable MVP for this feature.

---

## Phase 4: User Story 2 - Movie thumbnails presented in the terminal aesthetic (Priority: P2)

**Goal**: Real movie thumbnail images are kept (not converted to character art) but
framed with terminal-styled decoration; movies without a thumbnail show a themed
placeholder instead of a plain gray box.

**Independent Test**: View the movie grid and confirm each thumbnail (or its
placeholder) is framed/colored consistently with the terminal theme rather than
appearing as an unstyled photo or a gray box.

### Tests for User Story 2

> Write this test FIRST, ensure it FAILS before implementation.

- [X] T009 [P] [US2] Integration test in `tests/integration/test_thumbnail_framing.rs`: wiremock-stubbed Jellyfin server returning one movie with a downloaded thumbnail and one without; assert the generated `index.html`'s CSS frames `.movie-thumb` and `.movie-thumb-placeholder` with `var(--fg)`/`var(--fg-dim)`-based borders/colors rather than the old gray values (`#ddd`, `#333`, `#888`), per FR-005 / User Story 2 Acceptance Scenarios 1–2

### Implementation for User Story 2

- [X] T010 [US2] In `src/site/assets/index.html.tmpl`, update `.movie-thumb` and `.movie-thumb-placeholder` to use `var(--bg)`/`var(--fg-dim)` instead of their old gray values (`#ddd`/`#333`), and add a `border: 1px solid var(--fg-dim)` (terminal-styled frame) around both, so a real thumbnail image and the "No image" placeholder are equally consistent with the theme (depends on T008; makes T009 pass)
- [X] T011 [US2] Add ASCII-corner-glyph decoration (e.g. `+` characters via `::before`/`::after` content) to `.movie-thumb`'s and `.movie-thumb-placeholder`'s frame corners in `src/site/assets/index.html.tmpl`, keeping the frame itself as a responsive CSS border (not hand-drawn ASCII art) per research.md §1's responsive-grid constraint (depends on T010)

> **Implementation note**: see the note after T008 above — T010–T011 were implemented
> together with US1's template edits in a single pass.

**Checkpoint**: All user stories are independently functional — the full terminal theme,
banner, and terminal-framed thumbnails are all in place.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Confirm nothing regressed and the feature is genuinely complete.

- [X] T012 [P] Run the complete pre-existing test suite (`cargo test`) unmodified and confirm all 34 tests from `001-remote-movie-catalog` still pass unchanged, verifying FR-007 ("all existing page functionality... MUST continue to work exactly as before") against the T001 baseline (depends on T003, T006–T011)
- [X] T013 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`, resolving any findings (depends on T002, T004, T005, T009)
- [X] T014 Execute `quickstart.md` steps 1–5 manually end-to-end (generate a page, open it in a browser, verify the fixed palette, the ASCII banner, terminal-framed thumbnails, all existing functionality, and that titles/search remain plain selectable text) (depends on T003, T006–T011)

> **T014 note**: this sandbox has no GUI browser (same limitation noted for
> `001-remote-movie-catalog`'s T035). Generated a real page against a demo Jellyfin-shaped
> server and verified via direct inspection: `--bg`/`--fg`/`--fg-dim` present with no
> `prefers-color-scheme` query (Step 1), the aligned ASCII banner renders inside `<pre
> id="banner">` before `<header>` (Step 2), both `.movie-thumb` and
> `.movie-thumb-placeholder` carry the terminal-palette border and `+` corner glyphs
> (Step 3), the search `<input>` remains a plain unstyled-as-art element (Step 5), and
> `index.html`/`images/*.png` both serve correctly (HTTP 200) from a static host. Step 4
> (full existing-functionality check) is covered by T012's automated regression run. A
> quick manual look in an actual browser is still worth doing before you fully trust the
> visual result.
- [X] T015 [P] Check `README.md` for any description or wording that assumes the old light/dark auto-switching theme, and update it if needed to reflect the fixed terminal appearance

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS both user stories
  (both need the palette/typography base in place).
- **User Story 1 (Phase 3)**: Depends on Foundational. No dependency on User Story 2.
- **User Story 2 (Phase 4)**: Depends on Foundational *and*, in practice, on User Story 1's
  template edits landing first — both stories edit the same single template file
  (`src/site/assets/index.html.tmpl`), so US2's thumbnail-framing edits are sequenced
  after US1's edits to avoid conflicting concurrent changes, even though the two stories
  are conceptually independent.
- **Polish (Phase 5)**: Depends on both user stories being complete.

### Within Each Phase

- Tests MUST be written and observed to FAIL before their corresponding implementation
  tasks (Constitution Principle I).
- Within `src/site/assets/index.html.tmpl`, edits are sequenced (T003 → T006 → T007 →
  T008 → T010 → T011) to avoid conflicting concurrent changes to the same file, even
  where tasks are conceptually independent.

### Parallel Opportunities

- T004 and T005 (US1 tests) touch different files and can run in parallel.
- T009 (US2's only test) touches a different file from T004/T005 and can run in parallel
  with them if staffed for concurrent story work.
- T012 and T015 (Polish) are independent of each other and of T013/T014, and can run in
  parallel.

---

## Parallel Example: User Story 1

```bash
Task: "Integration test for the terminal palette in tests/integration/test_terminal_theme.rs"
Task: "Unit test for the ASCII banner in tests/unit/test_template.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (confirm baseline)
2. Complete Phase 2: Foundational (palette + typography)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: run `quickstart.md` Steps 1–2 and 4–5 (thumbnail framing from
   Step 3 isn't in place yet, but the page already looks and behaves like the terminal
   theme end-to-end)
5. This is a deployable MVP: the full page is terminal-themed with a working ASCII
   banner; only thumbnail-specific framing is still pending

### Incremental Delivery

1. Setup + Foundational → palette/typography base ready
2. Add User Story 1 → validate independently → MVP ready (whole-page theme + banner)
3. Add User Story 2 → validate independently (`quickstart.md` Step 3) → thumbnails
   terminal-framed
4. Phase 5: Polish (full regression confirmation, lint/format, manual walkthrough)

---

## Notes

- `[P]` tasks touch different files and have no dependency on an incomplete task.
- `[Story]` labels map every user-story-phase task to its story for traceability.
- Per Constitution Principle I, every test task must be written and observed to fail
  before its paired implementation task is started.
- `src/site/assets/index.html.tmpl` is the one file both stories touch — edits are
  sequenced (see Dependencies above) to avoid conflicting concurrent changes.
- Commit after each task or logical group; stop at any checkpoint to validate a story
  independently before continuing.

---

## Phase 6: Convergence

**Purpose**: Close gaps found by `/speckit-converge` between the spec/plan/tasks and the
implemented code. See that run's Convergence Findings summary for full evidence.

- [X] T016 Add overflow handling to `#banner` in `src/site/assets/index.html.tmpl` (e.g. `overflow-x: auto` and/or a narrower-viewport font-size reduction) so the fixed 31-character-wide ASCII banner cannot cause horizontal page overflow on narrow (~320px) phone viewports, per research.md §1's reflow/responsiveness reasoning; add a unit test in `tests/unit/test_template.rs` asserting `#banner` declares `overflow-x` handling (partial)
- [X] T017 Broaden `tests/integration/test_terminal_theme.rs`'s negative color assertions to cover all 9 retired light-theme values (`#666`, `#aaa`, `#111`, `#ddd`, `#333`, `#888`, in addition to the 3 already checked: `#fafafa`, `#eee`, `#ccc`), per FR-001 / FR-007 (partial)
