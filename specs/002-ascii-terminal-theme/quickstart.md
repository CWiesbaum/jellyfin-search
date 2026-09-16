# Quickstart: ASCII Terminal Theme

Validates that the generated page now presents as a black-background, green-text
terminal with an ASCII-art banner, while every previously-existing behavior still works.
This builds on `specs/001-remote-movie-catalog/quickstart.md` — run that guide's steps 1–2
first to produce a generated site; this guide covers what's new.

## Prerequisites

- Everything from `specs/001-remote-movie-catalog/quickstart.md`'s Prerequisites.
- A generated `./out` directory from that guide's steps 1–2 (a real export, or against a
  test Jellyfin instance/mock — any library with at least a couple of movies, ideally
  including at least one with a downloaded thumbnail and one without).

## 1. Confirm the fixed terminal palette

Serve `./out` as static files (as in `001-remote-movie-catalog`'s quickstart step 3) and
open it in a browser.

**Expected outcome**:
- The page background is black and all text (heading, search box, movie titles,
  metadata, "last updated") is green.
- This holds regardless of your OS/browser's light or dark appearance setting — toggle
  your system theme and confirm the page does **not** change (FR-006).

## 2. Confirm the ASCII banner

**Expected outcome**: A hand-authored ASCII-art banner is visible near the top of the
page, rendered in the same monospace green-on-black style as the rest of the page.

## 3. Confirm thumbnails are real images, terminal-framed

**Expected outcome**: Movies with a downloaded thumbnail show the actual poster image
(not character-art), framed with terminal-styled decoration (e.g., a bordered box)
consistent with the rest of the page. Movies without a thumbnail show a themed
placeholder, not a plain gray box (FR-005, User Story 2).

## 4. Confirm existing functionality is unaffected

Repeat `specs/001-remote-movie-catalog/quickstart.md`'s step 3 checks 1–5 (search
filters correctly, a "no match" state appears for an unknown title, the browser
devtools network panel shows zero requests to Jellyfin, and a "last updated" timestamp is
visible) — all MUST still pass, now inside the new visual theme (spec.md FR-007,
Success Criterion SC-002).

## 5. Confirm text content itself wasn't turned into decorative art

**Expected outcome**: Movie titles and the search input remain plain, normal, selectable
text — you can still select/copy a movie title and type normally into the search box.
Only the page's static chrome (banner, borders) uses ASCII-art styling (FR-003).
