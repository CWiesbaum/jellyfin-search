# Contract: Docker Image

This is the published Docker image's external interface to whoever pulls and runs it. It
wraps the existing CLI contract (`specs/001-remote-movie-catalog/contracts/cli-interface.md`)
without changing it — every behavior, flag, exit code, and stdout/stderr guarantee documented
there applies unchanged inside the container. Per Constitution Principle V, changes to this
contract follow semantic versioning, matching the image's own version tags (data-model.md's
Release Version / Image Tag).

## Image reference

```text
cwiesbaum/jellyfin-catalog-export:<tag>
```

## Tags

| Tag        | Kind     | Meaning                                                              |
|------------|----------|------------------------------------------------------------------------|
| `<X.Y.Z>`  | pinned   | The exact released version matching that version's `Cargo.toml`. Immutable once published — never reassigned to different image contents (FR-007). |
| `latest`   | floating | Always resolves to the newest stable released version's image.       |

## Platforms

A single tag resolves to one multi-architecture manifest covering both:

- `linux/amd64`
- `linux/arm64`

`docker pull`/`docker run` automatically select the correct architecture for the host; no
architecture-specific tag suffix is needed or provided (FR-003).

## Entrypoint

```dockerfile
ENTRYPOINT ["jellyfin-catalog-export"]
```

No default `CMD` — all of the CLI's own required flags (`--server-url`, `--output-dir`) must
be supplied as arguments to `docker run`, exactly as they would be to the native binary:

```bash
docker run --rm \
  -e JELLYFIN_API_KEY \
  --user "$(id -u):$(id -g)" \
  -v "$(pwd)/out:/data" \
  cwiesbaum/jellyfin-catalog-export:latest \
  --server-url http://jellyfin.local:8096 \
  --output-dir /data/site
```

`--user "$(id -u):$(id -g)"` overrides the image's default user (see Runtime user below)
with the invoking host user, so the process can write into `./out` — a bind-mounted host
directory's ownership has no relation to the image's default uid, and writing to it as a
mismatched uid fails with a permission error.

## Volumes

`--output-dir` MUST be given a path inside the container that is backed by a bind mount or
named volume, so generated output persists outside the container's writable layer once it
exits — but `--output-dir` itself MUST be a subdirectory *within* that mount (e.g.
`/data/site` above), not the mount point itself (e.g. not `/data`). The CLI's atomic publish
step (`specs/001-remote-movie-catalog/research.md`'s atomic-write approach) does
`rename(staging_dir, output_dir)`, and the kernel refuses to rename onto an active
bind-mount point (`EBUSY`) — only onto a plain subdirectory within one. The mount point
itself doesn't need to pre-exist in the image — Docker creates it automatically when
bind-mounting.

## Environment variables

- `JELLYFIN_API_KEY` — passed through to the CLI exactly as documented in the CLI contract
  (used when `--api-key` is not given).

## Exit codes

Identical to the native binary — see
`specs/001-remote-movie-catalog/contracts/cli-interface.md`'s Exit Codes table (`0`, `1`,
`2`, `3`). The container's exit code is the CLI process's exit code; nothing in the image
wraps or reinterprets it.

## Runtime user

The image runs as Alpine's built-in non-root `nobody` user (uid/gid `65534`) by default —
not `root`, and not a custom-created user (which would require a `RUN` step; see
research.md's build-time constraint that the Dockerfile never executes any
target-architecture code, not just avoiding compilation under QEMU). Running as non-root has
no effect on the CLI's documented behavior since the tool needs no elevated privileges. When
bind-mounting a host directory for `--output-dir` (see Volumes above), override this with
`--user "$(id -u):$(id -g)"` so the container process's uid matches the mounted directory's
owner.

## What this image does not do

Unchanged from the CLI contract's own "out of scope" section: no scheduling, no
publishing/hosting of `--output-dir`'s contents, no service lifecycle. The image is a
runnable packaging of the same one-shot CLI tool — nothing more.
