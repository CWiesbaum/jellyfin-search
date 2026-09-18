# Quickstart: Validating the Multi-Architecture Docker Image

This walks through validating the feature end-to-end locally, without waiting for a real
tagged release to exercise the CI workflow. See `contracts/docker-image-contract.md` for the
image's interface and `data-model.md` for the tag/version rules being checked.

## Prerequisites

- Docker with Buildx (`docker buildx version` succeeds).
- QEMU user-mode emulation registered, for running (not compiling) whichever architecture
  isn't native to your dev machine (e.g. `arm64` on an x86_64 host, or `amd64` on an ARM64
  host):
  ```bash
  docker run --privileged --rm tonistiigi/binfmt --install <arm64|amd64>
  ```
- Both target binaries cross-compiled and placed where the Dockerfile expects them (see
  research.md §1–2):
  ```bash
  ./scripts/build-cross-binaries.sh
  ```

## 1. Build each architecture and smoke-test it

```bash
./scripts/docker-smoke-test.sh linux/amd64
./scripts/docker-smoke-test.sh linux/arm64
```

**Expected outcome**: both invocations build/load a single-platform image, run
`--help` successfully, and run a full export against the existing `wiremock`-backed test
fixture, exiting `0`. Before the Dockerfile and `dist/` binaries exist, this script is
expected to fail — that failure is the "red" half of this feature's test-first cycle
(plan.md's Testing section).

## 2. Build the real multi-architecture manifest (without publishing)

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t cwiesbaum/jellyfin-catalog-export:local-test \
  .
```

**Expected outcome**: succeeds without `--push`, confirming both platforms build from the
same Dockerfile/context. (Buildx can't `--load` a multi-platform result locally — that's
what step 1's per-platform runs already verified individually.)

## 3. Inspect the image contents

```bash
docker buildx build --platform linux/amd64 --load -t jce:inspect .
docker run --rm --entrypoint sh jce:inspect -c 'cat /etc/os-release; which cc gcc rustc 2>&1 || true'
```

**Expected outcome**: `/etc/os-release` reports Alpine at the pinned version (research.md
§4); no compiler is found (confirms FR-005 — no build tooling leaks into the final image).

## 4. Validate version/tag rules (without touching real Docker Hub state)

```bash
./scripts/current-version.sh
./scripts/test-check-tag-not-published.sh
./scripts/test-check-version-matches-tag.sh
```

**Expected outcome**: `current-version.sh` prints `Cargo.toml`'s version (e.g. `0.2.0` — the
git tag you'd push to trigger a real publish is that value prefixed with `v`, e.g. `v0.2.0`);
both test scripts print `PASS` for each of their cases. These are the same checks
`.github/workflows/docker-publish.yml` runs on every tag push (research.md §3).

## 5. Full validation (real publish)

Actually publishing only happens by pushing a `v*.*.*` git tag, which runs
`.github/workflows/docker-publish.yml` end-to-end: pre-flight version/tag checks, both
architectures built and smoke-tested, then one atomic `docker buildx build --push`.
Confirm success by checking that `docker pull cwiesbaum/jellyfin-catalog-export:<version>`
and `:latest` both resolve on both an x86_64 and an ARM64 host (User Story 1's Independent
Test).
