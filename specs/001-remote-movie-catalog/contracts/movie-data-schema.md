# Contract: Embedded Catalog Data Schema

This is the JSON shape the generator embeds into `index.html` (Phase 0 §4) and that the page's
vanilla JS reads via `JSON.parse`. It is also, in effect, the tool's data contract — anything
built later that wants to read a generated snapshot (e.g. an alternate front-end) depends on
this shape. Per Constitution Principle V, changes here follow semantic versioning: adding an
optional field is MINOR-compatible; removing a field, changing a field's type, or changing how
an existing field is interpreted is a MAJOR (breaking) change.

## Location

A `<script id="catalog-data" type="application/json">…</script>` block inside the generated
`index.html`, containing exactly the JSON described below (no JavaScript wrapper, so it can be
parsed with a plain `JSON.parse(el.textContent)`).

## Shape

```json
{
  "generatedAt": "2026-09-16T14:32:00Z",
  "movies": [
    {
      "title": "Example Movie",
      "releaseYear": 2019,
      "genres": ["Drama", "Thriller"],
      "thumbnail": "images/a1b2c3.jpg"
    },
    {
      "title": "Another Movie With No Extra Metadata"
    }
  ]
}
```

## Field reference

| Field | Type | Always present? | Notes |
|---|---|---|---|
| `generatedAt` | string, ISO 8601 UTC | Yes | When this snapshot was generated. |
| `movies` | array | Yes | May be `[]` for an empty library. |
| `movies[].title` | string | Yes, per entry | Never empty (see data-model.md validation rules). |
| `movies[].releaseYear` | integer | No | Field is omitted entirely when unknown — consumers MUST NOT assume its presence. |
| `movies[].genres` | array of string | No | Field is omitted entirely when Jellyfin reported none; when present, is never an empty array (an empty list is omitted rather than included). |
| `movies[].thumbnail` | string (path, relative to `index.html`) | No | Field is omitted entirely when no thumbnail is available. Consumers MUST treat the path as relative to the document, not absolute or server-rooted. |

## Consumer expectations

- The generated page's own JS is the primary consumer and treats every optional field as
  possibly absent (falls back to a placeholder for missing year/genre/thumbnail, per the spec's
  edge case on incomplete metadata).
- Ordering of `movies` is not part of the contract — consumers that care about display order
  (e.g. alphabetical) MUST sort client-side rather than relying on export order.
- The schema intentionally excludes any Jellyfin-internal identifiers (item IDs, image tags) —
  see data-model.md's `JellyfinMovieItem` note — so the published data can't be used to
  construct Jellyfin API calls.
