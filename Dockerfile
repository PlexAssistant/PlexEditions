# syntax=docker/dockerfile:1
FROM rust:1.99-alpine AS build
RUN apk add --no-cache build-base
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
      linux/arm64) echo aarch64-unknown-linux-musl ;; \
      *) echo x86_64-unknown-linux-musl ;; \
    esac > /tmp/target
RUN --mount=type=cache,id=cargo-registry,sharing=locked,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-target-${TARGETPLATFORM},target=/src/target \
    mkdir -p src && printf 'fn main() {}\n' > src/main.rs && \
    rustup target add "$(cat /tmp/target)" && \
    cargo build --release --locked --bin editions --target "$(cat /tmp/target)" && \
    rm -rf src
COPY locales ./locales
COPY src ./src
ARG VERSION=dev
ENV EDITIONS_VERSION=${VERSION}
# The stub build above may share a second with this layer's mtime, in
# which case cargo skips the rebuild and /out keeps the stub binary.
# Drop this crate's fingerprints so it always recompiles.
RUN --mount=type=cache,id=cargo-registry,sharing=locked,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-target-${TARGETPLATFORM},target=/src/target \
    cargo clean --release --target "$(cat /tmp/target)" -p plex-editions && \
    cargo build --release --locked --bin editions --target "$(cat /tmp/target)" && \
    mkdir -p /out && cp "target/$(cat /tmp/target)/release/editions" /out/

FROM alpine:3.22
ARG VERSION=dev
ARG REVISION=""
ARG CREATED=""
LABEL org.opencontainers.image.title="PlexEditions" \
      org.opencontainers.image.description="Write movie edition titles on a Plex server" \
      org.opencontainers.image.authors="PlexAssistant" \
      org.opencontainers.image.vendor="PlexAssistant" \
      org.opencontainers.image.url="https://github.com/PlexAssistant/PlexEditions" \
      org.opencontainers.image.documentation="https://github.com/PlexAssistant/PlexEditions" \
      org.opencontainers.image.source="https://github.com/PlexAssistant/PlexEditions" \
      org.opencontainers.image.licenses="GPL-3.0" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${REVISION}" \
      org.opencontainers.image.ref.name="${VERSION}" \
      org.opencontainers.image.created="${CREATED}"
RUN apk add --no-cache ca-certificates
COPY --from=build /out/editions /usr/local/bin/editions
ENTRYPOINT ["/usr/local/bin/editions"]
CMD ["new"]
