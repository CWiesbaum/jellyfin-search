# Feature Specification: ASCII Terminal Theme

**Feature Branch**: `002-ascii-terminal-theme`

**Created**: 2026-09-16

**Status**: Draft

**Input**: User description: "The generated Website should be presented in ascii art. As color scheme it should look like a terminal output with black background and green textcolor."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse and search inside a terminal-styled page (Priority: P1)

A user opens the generated movie overview and experiences it as a classic computer
terminal: black background, green monospace text, applied consistently across every part
of the page — header, search box, movie list, and status indicators — not just isolated
elements.

**Why this priority**: This is the core of the request — the page's entire visual
identity — and it delivers the requested look on its own, independent of how individual
movie thumbnails end up being handled.

**Independent Test**: Can be fully tested by opening the generated page and visually
confirming every visible area uses the black-background/green-text terminal styling, with
no light-background or non-green-text elements remaining anywhere on the page.

**Acceptance Scenarios**:

1. **Given** the generated page is opened, **When** it loads, **Then** the background is
   black and all text is rendered in green, consistently across the entire page.
2. **Given** the page is opened on a device or browser set to a light appearance
   preference, **When** the page renders, **Then** it still shows the black-background,
   green-text terminal theme (the theme does not follow the visitor's light/dark
   preference).
3. **Given** a user searches for a movie title, **When** they type into the search field,
   **Then** search continues to work exactly as before, styled consistently with the
   terminal theme.

---

### User Story 2 - Movie thumbnails presented in the terminal aesthetic (Priority: P2)

A user browsing the movie list sees each movie's poster artwork presented in a way that
is visually consistent with the surrounding terminal theme, rather than looking like an
out-of-place full-color photograph dropped into an otherwise monochrome page.

**Why this priority**: Reinforces visual consistency and the "ASCII art" look the request
specifically calls for, but the page is fully usable and already delivers the primary
request via User Story 1 alone.

**Independent Test**: Can be fully tested by viewing the movie grid and confirming
thumbnail representations are visually consistent with the terminal theme, and that
movies without a downloaded thumbnail show a themed placeholder rather than a generic
gray box.

**Acceptance Scenarios**:

1. **Given** a movie has a downloaded thumbnail, **When** it is displayed, **Then** the
   real image is shown, framed with terminal-styled decoration (e.g., a bordered box)
   consistent with the black-background/green-text theme.
2. **Given** a movie has no downloaded thumbnail, **When** it is displayed, **Then** a
   themed (terminal-style) placeholder is shown instead of a generic gray box.

---

### Edge Cases

- What happens to movie titles containing accented letters, non-Latin scripts, or other
  characters that don't map cleanly onto typical ASCII decorative art? Titles MUST always
  be rendered as their real, original text — never corrupted, transliterated, or replaced
  by decorative art — so they remain readable and searchable exactly as before.
- What happens to a movie with no thumbnail at all under the new theme? It MUST still
  show a clear, theme-consistent placeholder (per the existing "no image" edge case),
  not a broken or mismatched-looking element.
- How does the search "no match" state and the "library is empty" state look under the
  new theme? Both MUST remain clearly legible and MUST follow the same black-background,
  green-text styling as the rest of the page.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The generated page MUST present a black background with green foreground
  text across the entire page — header, search UI, movie list, status/placeholder
  messages, and the "last updated" indicator — not just selected sections.
- **FR-002**: The generated page MUST use a monospace font throughout, consistent with a
  terminal appearance.
- **FR-003**: Movie titles, the search input, and other textual content MUST remain
  rendered as normal, plain, selectable text — not stylized into large decorative
  ASCII-art banner lettering — so readability, searchability, and accessibility are
  unaffected by the visual restyle.
- **FR-004**: Decorative ASCII art (e.g., borders, dividers, box-drawing characters, a
  title banner) MAY be used around the page's structural chrome to reinforce the terminal
  aesthetic.
- **FR-005**: Each movie's thumbnail, when available, MUST be presented as its real
  downloaded image (not converted to character-based ASCII art), framed/bordered with
  terminal-styled decoration (e.g., ASCII box-drawing corners or a bordered box) so it
  reads as part of the overall terminal aesthetic rather than an unstyled photo dropped
  onto the page.
- **FR-006**: The terminal theme MUST apply consistently regardless of the visitor's
  operating system or browser light/dark appearance preference — this is a fixed visual
  identity for the page, not a toggleable or auto-switching theme.
- **FR-007**: All existing page functionality — search/filter by title, the "no match"
  state, the "library is empty" state, the "last updated" indicator, and graceful
  placeholders for movies with missing metadata — MUST continue to work exactly as
  before; this is a visual restyle only, not a behavior change.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A person opening the generated page immediately perceives a computer
  terminal aesthetic (black background, green text) within the first few seconds of
  viewing, with no configuration or action required on their part.
- **SC-002**: Users can still find a specific movie via search and browse the full
  library exactly as quickly as before the visual change — the restyle introduces no
  additional steps and no loss of functionality.
- **SC-003**: 100% of movie entries, including those without a downloaded thumbnail,
  render with a complete, non-broken, theme-consistent visual treatment (no broken
  images, no mismatched light-themed elements).

## Assumptions

- The terminal theme replaces the page's current color scheme outright; it is not an
  optional or user-toggleable alternate theme, and it does not vary with the visitor's
  system light/dark preference.
- "ASCII art" governs decorative and structural visual elements (borders, dividers, a
  title banner, thumbnail framing) rather than the thumbnail images themselves, which
  stay as real photos per FR-005; it does not apply to titles, the search input, or
  other textual content, which remain normal readable and searchable text per FR-003.
- The classic green-on-black terminal palette is treated as sufficiently readable on its
  own merits (it is the traditional monochrome-terminal look and is inherently
  high-contrast); no additional accent colors are introduced.
- This is a presentation-only change to the previously-implemented Remote Movie Library
  Overview feature: the underlying export pipeline (Jellyfin API interaction, CLI flags,
  output file structure, embedded JSON data schema) is unaffected.
