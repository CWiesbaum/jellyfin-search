# Quickstart: Remote Movie Library Overview

Validates the feature end-to-end: exporting a Jellyfin movie library and confirming the
generated site is genuinely usable with no connection to Jellyfin or the home network. See
`contracts/cli-interface.md` for full flag/exit-code reference and `data-model.md` /
`contracts/movie-data-schema.md` for the data shapes involved.

## Prerequisites

- Rust stable toolchain installed (`cargo --version`).
- Network access from this machine to a Jellyfin server (a real one, or a local test instance)
  that has at least a few movies in a library, for a realistic check.
- A Jellyfin API key with read access to that library (Jellyfin dashboard → API Keys).

## 1. Build the tool

```bash
cargo build --release
```

## 2. Run an export

```bash
export JELLYFIN_API_KEY="<your-api-key>"
./target/release/jellyfin-catalog-export \
  --server-url "http://<jellyfin-host>:8096" \
  --output-dir ./out
```

**Expected outcome**: exits `0`; stderr shows progress (movie count fetched, any
skipped/warning entries); stdout prints one summary line; `./out` now contains `index.html`, an
`images/` directory with thumbnail files, and nothing else related to Jellyfin credentials.

## 3. Confirm the output is self-contained

Serve the output directory as plain static files (simulating whatever hosts it in production —
no special server logic required):

```bash
cd out && python3 -m http.server 8000
```

Open `http://localhost:8000` in a browser **with your Jellyfin server's network temporarily
blocked, or simply on a different network/device than Jellyfin** (e.g. phone on mobile data
if Jellyfin is only on the home LAN), and:

1. Confirm the full movie list renders with title and, where available, year/genre/thumbnail
   (User Story 2).
2. Type part of a known movie title into the search field and confirm it filters to matching
   results (User Story 1, Acceptance Scenario 1).
3. Search for a title you know is *not* in the library and confirm a clear "no match" state is
   shown (User Story 1, Acceptance Scenario 2).
4. Open the browser's network/devtools panel while browsing and searching, and confirm **no
   requests are made to the Jellyfin server** — only the initial page load and its own
   image/asset files (FR-006).
5. Confirm a "last updated" timestamp is visible somewhere on the page (User Story 3,
   Acceptance Scenario 2).

## 4. Confirm resilience to a failed run

With `./out` already populated from step 2, re-run the tool against an unreachable/invalid
server URL:

```bash
./target/release/jellyfin-catalog-export \
  --server-url "http://does-not-resolve.invalid:8096" \
  --output-dir ./out
```

**Expected outcome**: exits `2`; stderr explains the connection failure; `./out` is
**unchanged** — still serving the snapshot from step 2 (validates FR-009 / Phase 0 §8's
atomic-write guarantee).

## 5. Confirm freshness on a real change

Add or remove a movie in the Jellyfin library used above, re-run the command from step 2, and
confirm the change is reflected in `./out/index.html`'s embedded data and the browser view
after a refresh (User Story 3, Acceptance Scenario 1).
