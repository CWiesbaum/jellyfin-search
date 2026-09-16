# jellyfin-catalog-export

A Rust CLI tool that exports a Jellyfin server's movie library and generates a
self-contained static site — a browsable, searchable movie catalog you can host anywhere,
with no connection back to Jellyfin or the home network required at view time.

See `specs/001-remote-movie-catalog/` for the full spec, plan, and design decisions behind
this tool.

## What it does

- Connects to a Jellyfin server's REST API and reads its movie library (title, release
  year, genres, thumbnail).
- Downloads each movie's thumbnail as a local image file.
- Renders a single static `index.html` (with vanilla JS for client-side search/filter) and
  the downloaded images into an output directory, styled as a black-background,
  green-text terminal with an ASCII-art banner — see
  `specs/002-ascii-terminal-theme/` for that presentation's spec, plan, and design
  decisions.
- Writes that output atomically: a failed run never touches a previously generated,
  already-published output directory.

## What it deliberately does not do

- No playback/streaming — this is an inventory-lookup tool only.
- No library management (adding, deleting, editing metadata).
- No scheduling: running this tool repeatedly (e.g. via cron or a systemd timer) is left
  to whatever invokes it.
- No publishing/hosting: getting the generated output directory onto a public web host is
  also left to whatever invokes it.

## Build

```bash
cargo build --release
```

The binary is written to `target/release/jellyfin-catalog-export`.

## Run

```bash
export JELLYFIN_API_KEY="<your-jellyfin-api-key>"
jellyfin-catalog-export \
  --server-url "http://<jellyfin-host>:8096" \
  --output-dir ./out
```

### CLI flags

| Flag | Required | Description |
|---|---|---|
| `--server-url <URL>` | Yes | Base URL of the Jellyfin server to export from. |
| `--api-key <KEY>` | No* | Jellyfin API key. *Required unless `JELLYFIN_API_KEY` is set. |
| `--output-dir <PATH>` | Yes | Directory to write the generated static site into. |
| `--library-name <NAME>` | No | Selects a specific movie library, if the server has more than one. Defaults to the first movie library found. |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success. |
| `1` | Configuration/argument error (missing flag, no API key, invalid `--server-url`). |
| `2` | Jellyfin connection or authentication error. |
| `3` | Output write error. |

See `specs/001-remote-movie-catalog/contracts/cli-interface.md` for the full contract,
including the stdout/stderr behavior.

## Testing

```bash
cargo test
```

Contract and unit tests run against fixtures/pure functions; integration tests spin up an
in-process mock Jellyfin server (`wiremock`) and run the compiled binary as a subprocess
against it, so no real Jellyfin instance is needed.

## Deploying the output

`--output-dir`'s contents are plain static files — serve them with any static file host
(a CDN, object storage with static hosting, a simple web server, etc.). The generated page
makes no runtime requests back to Jellyfin or any dynamic backend; it only loads its own
static assets (the page itself and the downloaded thumbnail images).
