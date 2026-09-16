# Phase 0 Research: ASCII Terminal Theme

All Technical Context items were resolvable from the feature spec plus the constraints
already established by `001-remote-movie-catalog`. This document records the decisions
and their rationale.

## 1. Scope of literal ASCII art: static banner only, not per-movie framing

**Decision**: Genuine hand-authored ASCII art (character-based line art) is applied only
to a single static element — a banner near the top of the page — rendered as
preformatted (`<pre>`) monospace text. Dynamically-sized, responsive elements (the search
box, the movie grid, individual thumbnail frames) are styled with CSS borders/outlines
using terminal-evocative characters (`+`, `-`, `|`) as corner/edge decoration rather than
full hand-drawn ASCII boxes.

**Rationale**: Hand-authored ASCII art is a fixed character grid — it cannot reflow. The
movie grid inherited from `001-remote-movie-catalog` is responsive
(`grid-template-columns: repeat(auto-fill, minmax(140px, 1fr))`), rendering a different
number of columns at different viewport widths, and individual movie titles vary in
length. Wrapping every card in literal ASCII-art borders would either break at
non-reference viewport widths or require runtime ASCII-art layout generation in
JavaScript — real complexity with no corresponding requirement (FR-004 says decorative
ASCII art "MAY" be used, it is not mandated everywhere). Confining literal art to one
static, fixed-content banner delivers the explicit "ASCII art" look the spec asks for
while keeping every dynamic element simple, responsive, and consistent with Constitution
Principle II (Simplicity & YAGNI).

**Alternatives considered**: Runtime image-to-ASCII conversion for thumbnails — explicitly
rejected by the user during `/speckit-clarify` (Option B was chosen over Option A).
Full ASCII-art card borders redrawn per breakpoint — rejected as significant, fragile
complexity for a decorative effect achievable more simply with CSS borders.

## 2. Fixed dark palette replaces the light/dark auto-switching theme

**Decision**: Remove the `@media (prefers-color-scheme: dark)` block and the
`color-scheme: light dark` declaration from `001-remote-movie-catalog`'s template. Set
`color-scheme: dark` on `:root` (so native form controls/scrollbars render dark-appropriate
too) and make the black-background/green-text palette the page's only appearance,
regardless of the visitor's OS/browser preference.

**Rationale**: Directly implements FR-006 ("the terminal theme MUST apply consistently
regardless of the visitor's operating system or browser light/dark appearance
preference... a fixed visual identity, not a toggleable or auto-switching theme"). Keeping
the old light-mode base styles around as dead, unreachable CSS would violate the
Constitution's Quality Standards ("dead code... MUST NOT be merged... if it is not used,
it is removed, not disabled").

**Alternatives considered**: Keeping the terminal theme as an *additional* alternate theme
selected via `prefers-color-scheme` (i.e., terminal-for-dark-mode-visitors,
original-light-theme otherwise) — rejected because FR-006 explicitly calls for one fixed
identity, not a conditional one.

## 3. Color values: single hue, two brightness levels for hierarchy

**Decision**: One green hue (`#33ff33`, a common "phosphor green" terminal color, softer
than pure `#00ff00` for long text runs) for primary text and headings; the same hue at
roughly half brightness (`#1a8c1a`) for secondary/meta text (the "last updated" indicator,
movie metadata line, empty-state messages) — replacing the gray-based hierarchy
`001-remote-movie-catalog` used for the same purpose. Background stays pure black
(`#000000`).

**Rationale**: Satisfies "black background and green textcolor" literally, while
preserving the existing design's visual hierarchy (primary vs. secondary text) using
brightness rather than switching hue, so the page doesn't need a second accent color that
would dilute the monochrome terminal look.

**Alternatives considered**: A single flat green for all text — rejected because it
flattens the hierarchy the current design relies on (e.g., distinguishing a movie's title
from its year/genre line). Multiple distinct hues (e.g., amber warnings, cyan links) —
rejected as unrequested scope beyond "green textcolor."

## 4. Typography: system monospace stack only, no web fonts

**Decision**: `ui-monospace, "Courier New", Menlo, Consolas, monospace` as the page's
sole font stack (replacing the current `system-ui, -apple-system, sans-serif` stack),
applied via `body`. No `@font-face` or external font request is introduced.

**Rationale**: A monospace font is essential to the terminal look (FR-002) and to any
ASCII-art content rendering correctly (ASCII art depends on every character occupying the
same width). Using only fonts already present on essentially every platform preserves
`001-remote-movie-catalog`'s zero-external-request guarantee (FR-006 there, carried
forward as a Constraint here) without adding a font file to ship or a CDN request to make.

**Alternatives considered**: A bundled/embedded web font (e.g., base64-inlined) for a more
authentic terminal glyph set — rejected as unnecessary weight and complexity for a
cosmetic preference; system monospace fonts are already visually convincing for this
purpose.

## 5. Testing strategy for the restyle

**Decision**: Add assertions to `tests/unit/test_template.rs` that the rendered HTML (a)
contains the fixed dark palette's color values, (b) does **not** contain a
`prefers-color-scheme` media query (proving the old conditional theme was removed, not
just supplemented), (c) declares a monospace font-family, and (d) contains the ASCII
banner's marker content. Run the complete pre-existing test suite (all contract,
integration, and unit tests from `001-remote-movie-catalog`) unmodified as the primary
regression gate for this spec's FR-007.

**Rationale**: Points (a)-(d) are the testable surface of this feature's own new
requirements; the unmodified existing suite is the cheapest, most direct way to verify
"all existing page functionality... MUST continue to work exactly as before" without
duplicating tests that already exist.

**Alternatives considered**: Visual/screenshot regression testing — rejected as new
tooling/infrastructure disproportionate to a CSS change, and not available in this
project's dependency set (no headless-browser dependency exists or is otherwise
justified by any other requirement).
