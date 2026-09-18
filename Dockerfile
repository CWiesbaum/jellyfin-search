# syntax=docker/dockerfile:1
#
# Single-stage image: the binary is cross-compiled outside Docker (see
# scripts/build-cross-binaries.sh and specs/003-docker-image-publish/research.md §1-2), so
# this Dockerfile only copies the correct prebuilt static binary for the platform being
# built. Deliberately has no RUN step: COPY and USER are builder-side operations that don't
# execute any target-architecture code, so cross-building never needs QEMU for anything —
# not just avoiding the rustc-under-QEMU segfault, but any code execution at all. Runs as
# Alpine's built-in "nobody" user (uid/gid 65534) rather than a custom-created one, since
# creating one would require a RUN step.
FROM alpine:3.20

ARG TARGETARCH

COPY dist/${TARGETARCH}/jellyfin-catalog-export /usr/local/bin/jellyfin-catalog-export

USER nobody

ENTRYPOINT ["/usr/local/bin/jellyfin-catalog-export"]
