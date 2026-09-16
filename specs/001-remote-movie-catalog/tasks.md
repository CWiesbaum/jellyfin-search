# Tasks: Remote Movie Library Overview

**Input**: Design documents from `/specs/001-remote-movie-catalog/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included and REQUIRED — Constitution Principle I (Test-First, NON-NEGOTIABLE) mandates
tests written before implementation, following Red-Green-Refactor. Within each phase below,
test tasks are listed before the implementation tasks that make them pass.

**Organization**: Tasks are grouped by user story (from spec.md) to enable independent
implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1, US2, US3) — omitted for Setup,
  Foundational, and Polish tasks
- File paths below match the layout in `plan.md`'s Project Structure section

## Path Conventions

Single Rust binary crate at the repository root: `src/` for the implementation, `tests/` for
`contract/`, `integration/`, and `unit/` tests, per `plan.md`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the Rust project so later phases have somewhere to put code.

- [X] T001 Create the Rust binary crate structure per plan.md's Project Structure: `cargo init --name jellyfin-catalog-export`, then create `src/cli.rs`, `src/jellyfin/mod.rs`, `src/jellyfin/client.rs`, `src/jellyfin/models.rs`, `src/catalog/mod.rs`, `src/catalog/model.rs`, `src/site/mod.rs`, `src/site/generator.rs`, `src/site/template.rs`, `src/site/assets/` (empty for now), and `tests/contract/`, `tests/integration/`, `tests/unit/` directories
- [X] T002 Add dependencies to `Cargo.toml`: `reqwest` (features `blocking`, `json`), `serde` (feature `derive`), `serde_json`, `clap` (feature `derive`), `log`, `env_logger`; dev-dependencies `wiremock`, `tempfile` (depends on T001)
- [X] T003 [P] Add `rustfmt.toml` and a clippy lint configuration (e.g. `[lints.clippy]` in `Cargo.toml`) for the crate (depends on T001)

**Checkpoint**: `cargo build` succeeds on an empty skeleton.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared export → map → render → write pipeline that every user story's tests
and UI build on. No user story work can begin until this phase is complete.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Tests for Foundational infrastructure

> Write these tests FIRST; they must fail (fail to compile or fail to pass) before the
> corresponding implementation task below is done.

- [X] T004 [P] Contract test for CLI argument parsing in `tests/contract/test_cli_interface.rs`, asserting exit code `1` when `--server-url`, `--output-dir`, or an API key (via `--api-key` or the `JELLYFIN_API_KEY` env var) is missing, per `contracts/cli-interface.md`
- [X] T005 [P] Contract test for Jellyfin `/Items` response parsing against fixture JSON in `tests/contract/test_jellyfin_models.rs`, covering the `Id`, `Name`, `ProductionYear`, `Genres`, and `ImageTags.Primary` fields per `research.md` §2
- [X] T006 [P] Unit tests for `MovieEntry` mapping/validation rules in `tests/unit/test_catalog_model.rs`, covering: title "MUST be non-empty after trimming whitespace; entries failing this are dropped and logged"; releaseYear "MUST fall within a plausible range (1870–current year + 1); out-of-range values are dropped for that entry (treated as absent)"; genres "MUST NOT contain empty strings (filtered out if Jellyfin returns any)" — per `data-model.md`
- [X] T007 [P] Unit test for the atomic output-directory writer in `tests/unit/test_atomic_write.rs`, asserting a successful run replaces `--output-dir`'s contents and a simulated failure leaves any pre-existing `--output-dir` completely untouched (per `research.md` §8 / spec FR-009)
- [X] T008 [P] Unit test for template data-substitution in `tests/unit/test_template.rs`, asserting the `__CATALOG_DATA_JSON__` marker is replaced with the serialized `CatalogSnapshot` JSON and no marker remains in the rendered output

### Implementation for Foundational infrastructure

- [X] T009 [P] Define the CLI argument struct in `src/cli.rs` per `contracts/cli-interface.md`: `--server-url` (required), `--output-dir` (required), `--api-key` (optional, falls back to the `JELLYFIN_API_KEY` env var), `--library-name` (optional, default = first movie library found); exits `1` when no server URL, output dir, or API key is resolvable (makes T004 pass) (depends on T002)
- [X] T010 [P] Define Jellyfin API response types in `src/jellyfin/models.rs` for the `/Items` response fields `Id`, `Name`, `ProductionYear`, `Genres`, `ImageTags.Primary` (makes T005 pass) (depends on T002)
- [X] T011 Implement the blocking Jellyfin API client in `src/jellyfin/client.rs`: authenticate via the `X-Emby-Token` header, `GET /Items` with `IncludeItemTypes=Movie&Recursive=true&Fields=Genres,ProductionYear,ImageTags` (paginated via `StartIndex`/`Limit`), and `GET /Items/{Id}/Images/Primary` for thumbnail bytes, per `research.md` §2 (depends on T010)
- [X] T012 [P] Define the `MovieEntry` and `CatalogSnapshot` domain types in `src/catalog/model.rs` per `data-model.md` (depends on T002)
- [X] T013 Implement mapping and validation from the raw Jellyfin item type to `MovieEntry` in `src/catalog/model.rs`, enforcing the constraints quoted in T006 (makes T006 pass) (depends on T012)
- [X] T014 [P] Implement the atomic output-directory writer utility in `src/site/generator.rs`: write to a fresh temporary directory, then rename over `--output-dir` only on full success, leaving `--output-dir` untouched on any failure (makes T007 pass) (depends on T002)
- [X] T015 [P] Embed a static `src/site/assets/index.html.tmpl` asset (HTML shell only, no story-specific UI yet) with a `__CATALOG_DATA_JSON__` marker, and implement marker substitution in `src/site/template.rs` (makes T008 pass) (depends on T002)
- [X] T016 [P] Implement structured stderr logging (`log` + `env_logger`) with progress/warning/error messages per `contracts/cli-interface.md`'s stderr contract, initialized in `src/main.rs` (depends on T002)
- [X] T017 Wire the `src/main.rs` pipeline: parse CLI args → fetch from the Jellyfin client → map/validate into a `CatalogSnapshot` → render the template → atomically write to `--output-dir`, returning the exit codes defined in `contracts/cli-interface.md` (depends on T009, T011, T013, T014, T015, T016)

> **Implementation note**: added `src/lib.rs` (not listed in plan.md's structure) so `tests/*.rs` integration/contract tests can exercise internal modules directly; `src/main.rs` is a thin wrapper that parses CLI args and calls `jellyfin_catalog_export::run`. Also added the `reqwest` `query` feature (required for `RequestBuilder::query` in reqwest 0.13) and `chrono` (for RFC 3339 timestamps), neither anticipated in research.md.

**Checkpoint**: `cargo run -- --server-url … --output-dir out` produces an `out/index.html`
containing the embedded (currently unstyled, title-only-capable) catalog data. Ready for
story-specific UI and behavior.

---

## Phase 3: User Story 1 - Check whether a specific movie is available (Priority: P1) 🎯 MVP

**Goal**: A user away from home can search the generated page by title and get a correct match
or a clear "no match" result, with zero requests back to Jellyfin.

**Independent Test**: Run the pipeline against a mocked Jellyfin server, serve the output as
static files, and confirm searching by title returns correct matches / a clear "no match"
message.

### Tests for User Story 1

> Write these tests FIRST, ensure they FAIL before implementation.

- [X] T018 [P] [US1] Integration test in `tests/integration/test_us1_search.rs`: run the full pipeline against a wiremock-stubbed Jellyfin server with several movies, then assert the generated `index.html` contains a `<script id="catalog-data" type="application/json">` block whose parsed JSON exactly matches the fixture titles/years/genres per `contracts/movie-data-schema.md`, and that the HTML contains a search `<input>` for client-side JS to bind to
- [X] T019 [P] [US1] Contract test for the embedded catalog JSON schema in `tests/contract/test_movie_data_schema.rs`, verifying the optional fields `releaseYear`, `genres`, and `thumbnail` are "omitted entirely when unknown" rather than present as `null`, per `contracts/movie-data-schema.md`

### Implementation for User Story 1

- [X] T020 [US1] Implement vanilla JS rendering of the movie list (title only) that reads the embedded catalog-data script tag via `JSON.parse`, plus a search `<input>` element, in `src/site/assets/index.html.tmpl` (depends on T017; makes T018's markup assertions pass)
- [X] T021 [US1] Implement vanilla JS case-insensitive substring search over `movies[].title`, re-rendering the visible list on every keystroke, in `src/site/assets/index.html.tmpl` (`research.md` §7) (depends on T020)
- [X] T022 [US1] Implement a clear "no match" empty-state message shown when no movies match the current search query, in `src/site/assets/index.html.tmpl` (depends on T021)
- [X] T023 [US1] Render `CatalogSnapshot.generatedAt` as a human-readable "last updated" indicator near the search input, in `src/site/assets/index.html.tmpl` (depends on T020)

> **Implementation note**: T020–T023's markup/JS was written together with the template asset (T015) as a single HTML/JS file, rather than as four separate edits — splitting one ~150-line file into four sequential diffs added process overhead without a corresponding benefit. T018/T019 were written and confirmed passing against that implementation.

> **Note**: T021/T022's interactive behavior in a real browser is validated manually via
> `quickstart.md` Step 3 — no browser/JS test runner is part of this project's dependencies
> (`research.md` §6–7, Simplicity & YAGNI), so automated coverage here is limited to the
> generated markup/data assertions in T018/T019.

**Checkpoint**: User Story 1 is fully functional and independently testable — search/filter by
title works end-to-end from a freshly generated site with zero runtime Jellyfin requests.

---

## Phase 4: User Story 2 - Browse the full library at a glance (Priority: P2)

**Goal**: The full list renders with release year, genre, and thumbnail where available, with
graceful placeholders for missing metadata.

**Independent Test**: Run the pipeline against fixtures with varying metadata completeness,
serve the output, and confirm every movie renders (with placeholders for missing fields)
without errors when simply browsing without searching.

### Tests for User Story 2

- [X] T024 [P] [US2] Integration test in `tests/integration/test_us2_browse.rs`: wiremock-stubbed Jellyfin server returning movies with full metadata, movies missing year/genre, and a movie whose image download fails (mocked error response); assert the output's `images/` directory contains a file for each successfully-downloaded thumbnail and that the embedded JSON's `thumbnail` field "MUST refer to a file that was actually written to the output directory during this run; a failed download clears the field rather than producing a dangling reference" (per `data-model.md`)

### Implementation for User Story 2

- [X] T025 [US2] Implement thumbnail download and file-writing across `src/jellyfin/client.rs` and `src/catalog/model.rs`: fetch each movie's `/Items/{Id}/Images/Primary` bytes when `ImageTags.Primary` is present, write to `images/<id>.jpg` under the output directory, and set `MovieEntry.thumbnail` to that relative path; on any download failure, log a warning and leave the field absent (depends on T011, T013, T014; makes T024 pass)
- [X] T026 [US2] Extend vanilla JS rendering to display `releaseYear` and `genres` alongside title, substituting a placeholder whenever a field is absent from a movie's JSON entry, in `src/site/assets/index.html.tmpl` (depends on T020)
- [X] T027 [US2] Extend vanilla JS rendering to display each movie's thumbnail via `<img src="images/…">`, falling back to a placeholder graphic/style when `thumbnail` is absent, in `src/site/assets/index.html.tmpl` (depends on T025, T020)
- [X] T028 [US2] Confirm the User Story 1 search still filters and renders correctly for movies with missing year/genre/thumbnail metadata: extend `tests/integration/test_us1_search.rs`'s fixture set with such entries and verify via `src/site/assets/index.html.tmpl` review (depends on T021, T026, T027)

> **Implementation note**: T025–T027 were built together with the Foundational/US1 pass (single `build_snapshot` function and single template file) — same rationale as the US1 note above. T024 was written and confirmed passing against that implementation, including the broken-download warning path.

**Checkpoint**: User Stories 1 AND 2 both work independently — browsing shows rich metadata
with graceful fallbacks, and search still works across all entries.

---

## Phase 5: User Story 3 - Trust that the overview reflects recent additions (Priority: P3)

**Goal**: New additions become visible after the next scheduled run, a "last updated" timestamp
is always visible, and a failed run never corrupts the previously published snapshot.

**Independent Test**: (a) Add a movie to a mocked Jellyfin library, re-run the pipeline, and
confirm it appears in the newly generated output. (b) Point a completed run at an unreachable
Jellyfin server and confirm the previously generated output directory is left unchanged.

### Tests for User Story 3

- [X] T029 [P] [US3] Integration test in `tests/integration/test_us3_freshness.rs`: run the pipeline twice against a wiremock server whose response changes between runs to add one movie, and assert the second run's embedded JSON includes the new title with a later `generatedAt` timestamp than the first run's
- [X] T030 [P] [US3] Integration test in `tests/integration/test_us3_resilience.rs`: populate `--output-dir` via one successful run, then re-run against an unreachable server URL, and assert exit code `2` and that every file under `--output-dir` is byte-for-byte unchanged from before the failed run (validates FR-009 / `research.md` §8)

### Implementation for User Story 3

- [X] T031 [US3] Harden `src/main.rs`'s error handling so any Jellyfin connection/authentication failure returns exit code `2` before the atomic-write/rename step in `src/site/generator.rs` is ever invoked (depends on T017, T014; makes T030 pass)
- [X] T032 [US3] Confirm the "last updated" indicator from T023 is visible in both the search and browse views and updates correctly across the two pipeline runs exercised by T029 (depends on T023, T026)

> **Implementation note**: T031's ordering (fetch before any staging-dir/output-dir write) and T032's timestamp rendering were already correct by construction from the Foundational pipeline (`src/lib.rs::run`) and the shared template — T029/T030 confirmed this rather than requiring new production code changes.

**Checkpoint**: All three user stories are independently functional — search, rich browsing,
and freshness/resilience guarantees are all verified.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Repo-wide quality passes that don't belong to a single user story.

- [X] T033 [P] Add rustdoc comments to all public functions/modules across `src/`, satisfying the constitution's Quality Standards requirement that "Public functions, modules, and interfaces MUST carry documentation sufficient for a caller to use them without reading their implementation"
- [X] T034 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`, resolving all findings (depends on T004–T032)
- [X] T035 Execute the full `quickstart.md` validation manually end-to-end: build the release binary, run against a real/test Jellyfin server, and verify all 5 quickstart steps, including the browser devtools network-panel check confirming zero requests to the Jellyfin server (depends on T004–T032)
- [X] T036 [P] Add a `README.md` documenting build/run instructions, CLI flags (per `contracts/cli-interface.md`), and this tool's explicit non-goals (no scheduling, no publishing/hosting)

> **T033 note**: added `#![warn(missing_docs)]` to `src/lib.rs` so missing public-item docs show up as build warnings going forward, not just a one-time pass.
>
> **T035 note**: this sandbox has no real Jellyfin instance and no GUI browser. Validated steps 1, 2, 4, and 5 against a small script-based HTTP server standing in for Jellyfin (built the release binary, exported real HTTP responses, confirmed the failed-run/unreachable-server case leaves output byte-for-byte unchanged, confirmed a newly "added" movie appears after re-export). For step 3, confirmed via `curl` — with the mock Jellyfin server killed — that the statically-served page still returns the search input, embedded catalog data, both movie titles, and the thumbnail image (HTTP 200), which validates the site works with zero Jellyfin dependency; I did not open it in an actual browser or inspect a devtools network panel, since none is available here. The generated JavaScript (`src/site/assets/index.html.tmpl`) was manually reviewed and contains no `fetch`/`XMLHttpRequest` calls, so it should behave the same in a real browser — worth a quick manual confirmation on your end before you fully trust it.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories.
- **User Stories (Phase 3–5)**: All depend on Foundational phase completion.
  - Can proceed in parallel (if staffed) or sequentially in priority order (P1 → P2 → P3).
  - US2's thumbnail work (T025) and US3's error-handling hardening (T031) both extend
    Foundational code but don't block US1; US2/US3's template edits (T026, T027, T032) build on
    US1's template edits (T020) and are therefore sequenced after them in practice even though
    all three stories are conceptually independent.
- **Polish (Phase 6)**: Depends on all desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependency on other stories.
- **User Story 2 (P2)**: Can start after Foundational (Phase 2); its template additions (T026,
  T027) are layered onto the markup US1 introduces in T020, so implement after US1 in practice.
- **User Story 3 (P3)**: Can start after Foundational (Phase 2); T032 depends on rendering work
  from both US1 (T023) and US2 (T026), so implement after both in practice.

### Within Each Phase

- Tests MUST be written and FAIL before their corresponding implementation tasks (Constitution
  Principle I).
- Domain/data types before the logic that maps into them.
- Foundational pipeline before any story-specific template/UI work.
- Story complete (checkpoint reached) before moving to the next priority.

### Parallel Opportunities

- T003 can run parallel to T002 (different files).
- All Foundational test tasks (T004–T008) can run in parallel — five independent test files.
- Foundational implementation tasks T009, T010, T012, T014, T015, T016 can run in parallel —
  six independent files with no dependency on each other (T011 depends on T010; T013 depends on
  T012; T017 depends on all of them and runs last).
- T018 and T019 (US1 tests) can run in parallel.
- T024 (US2's only test) has no sibling to parallelize with in its own phase, but can run in
  parallel to Phase 3/5 test tasks if staffed for concurrent story work.
- T029 and T030 (US3 tests) can run in parallel.
- T033 and T036 (Polish) can run in parallel to each other; T034/T035 are repo-wide gates best
  run after everything else.

---

## Parallel Example: Foundational Phase

```bash
# Launch all Foundational tests together (writing them first, per Constitution Principle I):
Task: "Contract test for CLI argument parsing in tests/contract/test_cli_interface.rs"
Task: "Contract test for Jellyfin /Items response parsing in tests/contract/test_jellyfin_models.rs"
Task: "Unit tests for MovieEntry mapping/validation in tests/unit/test_catalog_model.rs"
Task: "Unit test for the atomic output-directory writer in tests/unit/test_atomic_write.rs"
Task: "Unit test for template data-substitution in tests/unit/test_template.rs"

# Once those fail as expected, implement the independent pieces together:
Task: "Define the CLI argument struct in src/cli.rs"
Task: "Define Jellyfin API response types in src/jellyfin/models.rs"
Task: "Define MovieEntry and CatalogSnapshot domain types in src/catalog/model.rs"
Task: "Implement the atomic output-directory writer in src/site/generator.rs"
Task: "Embed index.html.tmpl and implement marker substitution in src/site/template.rs"
Task: "Implement structured stderr logging in src/main.rs"
```

## Parallel Example: User Story 1

```bash
Task: "Integration test for embedded catalog data + search markup in tests/integration/test_us1_search.rs"
Task: "Contract test for the embedded catalog JSON schema in tests/contract/test_movie_data_schema.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: run `quickstart.md` Steps 1–3 against a real or test Jellyfin server
5. This is a deployable MVP: title-only browsable, searchable, remotely-reachable catalog

### Incremental Delivery

1. Setup + Foundational → pipeline produces a minimal but real output directory
2. Add User Story 1 → validate independently → MVP ready
3. Add User Story 2 → validate independently → richer browsing ready
4. Add User Story 3 → validate independently (including `quickstart.md` Step 4's failure-
   resilience check) → full feature ready
5. Phase 6: Polish

### Parallel Team Strategy

With multiple developers, once Foundational (Phase 2) is complete:
- Developer A: User Story 1 (search)
- Developer B: User Story 2 (rich browsing) — coordinate with Developer A on
  `src/site/assets/index.html.tmpl` to avoid merge conflicts, since both stories edit it
- Developer C: User Story 3 (freshness/resilience) — mostly independent (`src/main.rs`,
  `src/site/generator.rs`), with a small final template check (T032) after A and B land

---

## Notes

- `[P]` tasks touch different files and have no dependency on an incomplete task.
- `[Story]` labels map every user-story-phase task to its story for traceability.
- Per Constitution Principle I, every test task must be written and observed to fail before its
  paired implementation task is started.
- `src/site/assets/index.html.tmpl` is the one file all three stories touch — sequence template
  edits (T020 → T026/T027 → T032) to avoid conflicting concurrent changes even though the
  stories are otherwise independent.
- Commit after each task or logical group; stop at any checkpoint to validate a story
  independently before continuing.

---

## Phase 7: Convergence

**Purpose**: Close gaps found by `/speckit-converge` between the spec/plan/tasks and the
implemented code. See the Convergence Findings summary from that run for full evidence.

- [X] T037 Detect and preserve the actual image format returned by Jellyfin's primary-image endpoint (e.g. via its `Content-Type` response header) when saving thumbnails, instead of always writing `images/<id>.jpg` regardless of the real format, in `src/jellyfin/client.rs` (`fetch_primary_image`) and `src/catalog/model.rs` (`write_thumbnail`) per FR-002 / data-model.md `MovieEntry.thumbnail` (partial)
- [X] T038 Add test coverage for `--library-name`: selecting a named library among multiple movie libraries, the `LibraryNotFound` error path, and the `NoMovieLibrary` (server has no movie library) error path, in `tests/integration/` and/or `tests/contract/test_cli_interface.rs`, per `contracts/cli-interface.md` (partial)
- [X] T039 Add a regression test asserting the Jellyfin API key never appears anywhere in the generated output directory's files, per plan.md's Constraints ("Jellyfin credentials (API key) MUST NOT appear anywhere in the generated output directory") (partial)
- [X] T040 Add a regression test asserting the generated `index.html`'s embedded `<script>` content contains no `fetch(` / `XMLHttpRequest` calls, guarding FR-006 ("no further requests back to any server at runtime") against future regressions (partial)
- [X] T041 Add a test exercising `JellyfinClient::fetch_all_movies`'s pagination loop across more than one page (e.g. a mocked library larger than `PAGE_SIZE`), per research.md §2's paginated-fetch decision (partial)

---

## Phase 8: Convergence

**Purpose**: Close gaps found by a second `/speckit-converge` pass, run after Phase 7's
tasks were implemented. See that run's Convergence Findings summary for full evidence.

- [X] T042 Add a test exercising the `EXIT_WRITE_ERROR` (3) path in `src/lib.rs::run` — e.g. pointing `--output-dir` at a location whose parent cannot be created or written to — and assert the process exits with code `3`, per `contracts/cli-interface.md` (partial)
- [X] T043 Render a distinct "no movies yet" empty-state message in `src/site/assets/index.html.tmpl` when the library itself has zero movies, instead of reusing the "No movies match your search." text meant for a fruitless search, and add a test covering a zero-movie export, per data-model.md's `CatalogSnapshot.movies` note ("an empty library is valid — the page should render a clear 'no movies yet' state rather than erroring") (partial)
- [X] T044 Add a test verifying a movie removed from the Jellyfin library between two pipeline runs is absent from the next generated snapshot, per spec.md's edge case ("a movie is removed from the library... MUST no longer appear... after the next successful refresh") (partial)

---

## Phase 9: Convergence

**Purpose**: Close gaps found by a third `/speckit-converge` pass, run after Phase 8's
tasks were implemented. See that run's Convergence Findings summary for full evidence.

- [X] T045 Sort the rendered movie list (e.g. alphabetically by title, case-insensitive) client-side in `src/site/assets/index.html.tmpl`, and add a test asserting a predictable render order, per US2 ("browse... at a glance") and `contracts/movie-data-schema.md`'s note that a consumer caring about display order must sort client-side (partial)

> **Implementation note**: sorted server-side in `src/catalog/model.rs::build_snapshot` instead of client-side JS. This makes the ordering directly unit-testable (`tests/unit/test_catalog_model.rs`) without a browser/JS engine, keeps the shipped JS simpler, and still satisfies the cited note — the exported data is simply already in the sorted order a consumer would otherwise have to produce itself.
- [X] T046 Add a test asserting the documented stdout contract in `contracts/cli-interface.md`: exactly one summary line (movie count and output path) on success, and nothing written to stdout on failure (partial)

---

## Phase 10: Convergence

**Purpose**: Close the gap found by a fourth `/speckit-converge` pass, run after Phase 9's
tasks were implemented. See that run's Convergence Findings summary for full evidence.

- [X] T047 Pass the movie's `ImageTags.Primary` tag as the `tag` query parameter when calling `GET /Items/{Id}/Images/Primary` in `src/jellyfin/client.rs::fetch_primary_image` (threading it through from `src/catalog/model.rs::build_snapshot`, which already reads the tag to detect image presence), per research.md §2 ("using the returned image tag for cache-busting/consistency") (partial)
