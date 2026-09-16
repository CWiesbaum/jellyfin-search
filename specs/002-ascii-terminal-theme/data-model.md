# Phase 1 Data Model: ASCII Terminal Theme

This feature introduces no new data entities. `MovieEntry` and `CatalogSnapshot`, and the
embedded JSON schema they produce (`specs/001-remote-movie-catalog/contracts/movie-data-schema.md`),
are unchanged — this feature only changes how that already-existing data is *presented*.

The only "model" this feature adds is a small, fixed set of presentation tokens — the
terminal theme's palette and typography — which replace the equivalent values in
`001-remote-movie-catalog`'s template. They are recorded here for traceability, since
research.md §2–§4 explain the reasoning behind each value.

## Presentation Tokens

| Token | Value | Used for | Replaces |
|---|---|---|---|
| `--bg` | `#000000` | Page background (`body`) | `#fafafa` (light) / `#111` (dark media query) |
| `--fg` | `#33ff33` | Primary text: headings, movie titles, search input text | `#111` (light) / `#eee` (dark media query) |
| `--fg-dim` | `#1a8c1a` | Secondary text: "last updated", movie metadata line, empty-state messages | `#666` (light) / `#aaa` (dark media query) |
| Font stack | `ui-monospace, "Courier New", Menlo, Consolas, monospace` | All text on the page | `system-ui, -apple-system, sans-serif` |
| `color-scheme` | `dark` | `:root`, so native form controls/scrollbars match | `light dark` |

These are plain CSS custom properties declared once on `:root` in
`src/site/assets/index.html.tmpl` and referenced throughout the existing stylesheet — no
new build step or preprocessing is introduced.

## Lifecycle

None — these are static values baked into the template at development time, not data
computed or varied at runtime. Every generated page uses the same fixed palette
regardless of the exported library's contents, per FR-006 (fixed identity, not
conditional on visitor preference or data).
