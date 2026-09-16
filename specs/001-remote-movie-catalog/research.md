# Phase 0 Research: Remote Movie Library Overview

All Technical Context items were resolvable from the user-specified technical approach plus
established Jellyfin API and Rust ecosystem practices — no items remain marked
`NEEDS CLARIFICATION`. This document records the decisions and their rationale.

## 1. HTTP client: blocking `reqwest`, no async runtime

**Decision**: Use `reqwest` in blocking mode (`reqwest::blocking`) to talk to the Jellyfin REST
API, rather than the async client with a `tokio` runtime.

**Rationale**: The tool is a one-shot batch job invoked on a schedule by something external to
it (per the feature's explicit non-goal of owning scheduling). It fetches one library, item by
item, sequentially or with simple bounded concurrency at most — there is no need for an async
executor. Avoiding `tokio` removes an entire dependency tree and keeps the binary and its
mental model simple, per Constitution Principle II (Simplicity & YAGNI).

**Alternatives considered**: `reqwest` async + `tokio` — rejected as unnecessary complexity for
a sequential batch job. `ureq` (minimal sync HTTP client, no dependency on the `reqwest`/hyper
stack) — a reasonable lighter-weight alternative; `reqwest` blocking was chosen instead for its
more ergonomic JSON handling and wider ecosystem familiarity, but `ureq` remains a valid fallback
if binary size or dependency footprint becomes a concern later.

## 2. Jellyfin API surface used

**Decision**: Authenticate with an API key (sent as the `X-Emby-Token` header, Jellyfin's
supported mechanism), then:
- List movie items via `GET /Items` with `IncludeItemTypes=Movie`, `Recursive=true`, and
  `Fields=Genres,ProductionYear,ImageTags` to get title, year, genres, and the primary-image tag
  in one call (or paginated calls via `StartIndex`/`Limit` for large libraries).
- Fetch each movie's thumbnail via `GET /Items/{Id}/Images/Primary` (using the returned image
  tag for cache-busting/consistency), downloading the image bytes directly.

**Rationale**: This is Jellyfin's documented, stable REST surface for browsing a library and
retrieving primary images, and covers every field FR-001/FR-002 require (title, year, genre,
thumbnail) in a small, predictable number of requests.

**Alternatives considered**: Jellyfin's `/Users/{UserId}/Items` user-scoped endpoint — rejected
because the export doesn't need per-user data (watched state, favorites, etc.), just the library
catalog; the simpler `/Items` call avoids requiring a specific user context.

## 3. Credential handling

**Decision**: The Jellyfin API key is supplied via a `--api-key` CLI flag or, preferably, the
`JELLYFIN_API_KEY` environment variable (checked when `--api-key` is omitted), and is never
written into the generated output directory in any form.

**Rationale**: Environment-variable input avoids the credential appearing in shell history or
process listings (`ps`) the way a bare CLI argument would, while still allowing direct CLI use
for one-off runs or scripting flexibility. Keeping it out of the output directory is required
because the generated site (FR-010) is published with no access control.

**Alternatives considered**: Config file with the key on disk — rejected as an extra artifact to
keep out of version control/publishing for no benefit over an environment variable, for a
single-value secret.

## 4. Embedding catalog data into the generated page

**Decision**: The generated `index.html` embeds the movie catalog as a JSON literal inside an
inline `<script>` block (e.g. `<script id="catalog-data" type="application/json">…</script>`),
which the page's vanilla JS reads via `JSON.parse` on load — not via `fetch()` of a separate
JSON file.

**Rationale**: The feature spec (FR-006) and the user's technical approach both require that the
page perform no runtime requests back to any server; embedding the data inline guarantees the
page is fully self-contained and usable the instant the HTML document itself has loaded, with no
second request that could fail or be blocked.

**Alternatives considered**: `fetch()`-ing a sibling `data.json` at page load — rejected as an
unnecessary extra runtime request for data that is generated at the same time as the page and is
small enough to inline directly (a few thousand short JSON records).

## 5. Thumbnail images: local static files, not inlined base64

**Decision**: Downloaded thumbnails are written as individual static image files into the output
directory (e.g. `images/<movie-id>.jpg`) and referenced from the embedded catalog JSON by
relative path (`<img src="images/…">`); they are not base64-encoded into the HTML/JSON.

**Rationale**: "No further requests back to any server at runtime" (FR-006) is about not
depending on the Jellyfin server or a dynamic backend, not about the page making zero HTTP
requests of any kind — every HTML page with images makes such requests to its own static host,
same as it would for a stylesheet. Loading images as separate static files avoids inflating the
single HTML document to tens of megabytes for a multi-thousand-movie library (base64 has ~33%
overhead on top of already-large image bytes), which would work against SC-001/SC-004's
usability targets on typical mobile connections. The output remains "a ready-to-deploy set of
static files" as the user's technical approach describes, all served from the one static host.

**Alternatives considered**: Base64-inlining every thumbnail directly into the HTML/JSON —
rejected due to page-weight and load-time cost at the stated several-thousand-movie scale.
Omitting thumbnails entirely — rejected because FR-002 asks for them where available and they
materially help User Story 2 (browsing at a glance).

## 6. HTML/JS generation approach

**Decision**: Ship a static `index.html.tmpl` asset (HTML structure + vanilla JS for
render/search/filter) embedded in the binary via `include_str!`, with a single marker token
(e.g. `__CATALOG_DATA_JSON__`) that the generator replaces with the serialized catalog JSON at
run time via plain string substitution.

**Rationale**: A dedicated templating engine (Askama, Tera, Handlebars, …) buys nothing here —
there is exactly one dynamic value being injected into an otherwise-static document. Plain
substitution is simplest and dependency-free, matching Principle II.

**Alternatives considered**: Askama (compile-time typed templates) — rejected as unneeded
machinery for a single substitution point; would be reconsidered only if the page's structure
grows materially more dynamic.

## 7. Client-side search/filter behavior

**Decision**: Vanilla JS performs a case-insensitive substring match of the search input against
each movie's title, re-rendering the visible list on every keystroke (no debounce needed at the
stated scale of a few thousand in-memory records).

**Rationale**: Directly satisfies FR-003 (search/filter by title) with the simplest possible
client-side logic; a few thousand string comparisons per keystroke is trivially fast in a
browser, so no indexing/debouncing complexity is warranted.

**Alternatives considered**: A fuzzy-matching library — rejected as unnecessary complexity beyond
what FR-003 requires (title search), reconsidered only if user feedback asks for it later.

## 8. Failure handling & output atomicity

**Decision**: The tool writes its output to a fresh temporary directory first, and only replaces
the target output directory (e.g. via rename) once the full export-and-generate run has
succeeded. On any failure (Jellyfin unreachable, auth failure, partial data, write error), it
exits with a distinct non-zero code and leaves any previously-published output directory
untouched.

**Rationale**: Directly implements FR-009/edge case: "when the home server or its internet
connection is unreachable at a scheduled refresh time, the overview must keep serving the last
successfully synced data." Since publishing/hosting is outside this tool's scope, the guarantee
it owns is: never leave a partially-written or corrupt output for whatever republishes it.

**Alternatives considered**: Writing directly into the final output directory — rejected because
a run that fails partway through would leave a broken mix of old and new files.

## 9. Testing strategy

**Decision**: Use `wiremock` to run an in-process mock HTTP server standing in for Jellyfin in
integration tests, combined with fixture JSON files (captured/representative `/Items` responses)
for contract-level parsing tests, plus plain unit tests for pure logic (catalog mapping,
template substitution). All tests run via `cargo test` with no external Jellyfin instance
required.

**Rationale**: Satisfies Constitution Principle I (Test-First) in a way that keeps the suite
fast and hermetic (no real network/server dependency), so tests can genuinely be written and run
before implementation.

**Alternatives considered**: Testing only against a real, manually-run Jellyfin instance —
rejected as slow, non-reproducible, and impossible to run in CI.
