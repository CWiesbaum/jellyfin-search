# Phase 1 Data Model: Remote Movie Library Overview

Derived from the feature spec's Key Entities section and the Phase 0 research decisions. These
are the domain types the CLI tool maps Jellyfin API responses into, and the shape of what gets
embedded in the generated page. See `contracts/movie-data-schema.md` for the exact wire format.

## MovieEntry

Represents a single movie as shown in the overview (spec: "Movie Entry").

| Field | Type | Required | Notes |
|---|---|---|---|
| `title` | string | Yes | Movie title as reported by Jellyfin. Never empty — a movie with no title is skipped and logged as a warning (Observability), not included in the catalog. |
| `releaseYear` | integer or absent | No | Jellyfin's `ProductionYear`. Omitted (not `null`) when Jellyfin doesn't report one. If present, MUST be a plausible year (1870–current year + 1); an implausible value is treated as absent and logged as a warning rather than failing the run. |
| `genres` | array of string | No | Jellyfin's `Genres`. May be an empty array when Jellyfin reports none. |
| `thumbnail` | string (relative path) or absent | No | Relative path to the downloaded thumbnail file (e.g. `images/<id>.jpg`) under the output directory. Absent when Jellyfin has no primary image for the item, or the image download fails (logged as a warning; the rest of the entry is still included, per the edge case "movie missing some metadata still displays correctly"). |

No `id` field is required in the public/embedded data — client-side search/filter/render
operates purely on the fields above. An internal Jellyfin item ID is used only within the tool
(fetching images, log messages) and is not part of the published catalog contract.

**Validation rules** (enforced during mapping, before an entry is included in a `CatalogSnapshot`):
- `title` MUST be non-empty after trimming whitespace; entries failing this are dropped and
  logged, not included in the output (there is no reasonable way to search for or display an
  untitled entry).
- `releaseYear`, when present, MUST fall within a plausible range; out-of-range values are
  dropped for that entry (treated as absent) rather than failing the whole run.
- `genres`, when present, MUST NOT contain empty strings (filtered out if Jellyfin returns any).
- `thumbnail`, when present, MUST refer to a file that was actually written to the output
  directory during this run; a failed download clears the field rather than producing a
  dangling reference.

## CatalogSnapshot

Represents the full exported library as embedded in the generated page (spec: "Catalog
Snapshot").

| Field | Type | Required | Notes |
|---|---|---|---|
| `generatedAt` | string (ISO 8601 UTC timestamp) | Yes | When this export run completed. Displayed in the page as a "last updated" indicator (User Story 3, Acceptance Scenario 2). |
| `movies` | array of `MovieEntry` | Yes | May be empty (an empty library is valid — the page should render a clear "no movies yet" state rather than erroring). |

**Lifecycle**: A `CatalogSnapshot` is produced fresh on every tool run and is immutable once
written — there is no in-place update or partial-merge with a previous snapshot. A run either
produces a complete new snapshot (written atomically per Phase 0 §8) or fails entirely, leaving
the previous snapshot's output directory untouched. There are no state transitions within a
snapshot's lifetime; "freshness" is entirely expressed by comparing `generatedAt` to the current
time when a viewer looks at the page.

## JellyfinMovieItem (internal, not published)

The subset of Jellyfin's raw `/Items` response fields the tool reads, before mapping to
`MovieEntry`. Internal to `src/jellyfin/models.rs`; not part of any external contract, and may
change freely as the Jellyfin API surface used in Phase 0 §2 evolves.

| Field (Jellyfin API name) | Maps to |
|---|---|
| `Id` | used only to fetch the primary image; not published |
| `Name` | `MovieEntry.title` |
| `ProductionYear` | `MovieEntry.releaseYear` |
| `Genres` | `MovieEntry.genres` |
| `ImageTags.Primary` | presence triggers a thumbnail download; the tag value is not published |
