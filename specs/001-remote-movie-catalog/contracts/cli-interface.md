# Contract: CLI Interface

This is the tool's external interface to whatever invokes it (a cron job, a systemd timer, a CI
pipeline, a human at a terminal — all explicitly out of scope for this feature, per plan.md).
Per Constitution Principle V, changes to this contract follow semantic versioning: adding an
optional flag is MINOR/PATCH-compatible; removing or renaming a flag, changing default
behavior, or changing an exit code's meaning is a MAJOR (breaking) change.

## Invocation

```text
jellyfin-catalog-export --server-url <URL> --output-dir <PATH> [--api-key <KEY>] [--library-name <NAME>]
```

## Arguments

| Flag | Required | Default | Description |
|---|---|---|---|
| `--server-url <URL>` | Yes | — | Base URL of the Jellyfin server to export from (e.g. `http://jellyfin.local:8096`). Only needs to be reachable from wherever the tool runs — never from the published site's viewers. |
| `--api-key <KEY>` | No* | — | Jellyfin API key. *Required unless the `JELLYFIN_API_KEY` environment variable is set instead (see Phase 0 §3); if neither is provided, the tool exits with code 1. |
| `--output-dir <PATH>` | Yes | — | Local directory to write the generated static site into. Created if it does not exist. Replaced atomically only on a fully successful run (Phase 0 §8) — a failed run leaves any pre-existing directory at this path untouched. |
| `--library-name <NAME>` | No | first movie library found | Selects a specific Jellyfin movie library by name, for servers with more than one. |

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | Success — a complete new snapshot was generated and written to `--output-dir`. |
| `1` | Configuration/argument error (missing required flag, no API key available, invalid `--server-url`). Nothing is written or changed under `--output-dir`. |
| `2` | Jellyfin connection or authentication error (server unreachable, timed out, API key rejected). Nothing under `--output-dir` is changed — any previous snapshot there remains as-is. |
| `3` | Output write error (e.g. `--output-dir`'s parent is not writable, disk full). Nothing under `--output-dir` is changed. |

## stdout / stderr

- **stderr**: structured, human-readable log lines for progress and errors (e.g. "fetched 1,342
  movies", "warning: skipped movie with empty title", "error: could not connect to
  http://jellyfin.local:8096"). This is the tool's Observability surface (Constitution IV) — an
  operator or scheduler can diagnose a failure from these lines and the exit code alone.
- **stdout**: on success, a single final summary line (movie count and output path). Nothing is
  written to stdout on failure. stdout is intentionally kept quiet/parseable; stderr carries the
  diagnostic detail.

## Out of scope for this contract

Scheduling repeat runs, publishing/uploading `--output-dir`'s contents anywhere, and any
notion of "service" lifecycle (start/stop/restart) are explicitly not part of this tool, per the
feature's technical approach — the invoking environment owns all of that.
