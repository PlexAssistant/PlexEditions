default:
    @just --list

build:
    cargo build --release

test:
    cargo test

lint:
    cargo clippy --all-targets -- -D warnings

check:
    cargo fmt --all --check
    cargo clippy --all-targets -- -D warnings
    cargo test

fmt:
    cargo fmt

image:
    docker build --build-arg VERSION=`git describe --tags --always --dirty 2>/dev/null || echo dev` \
        --build-arg REVISION=`git rev-parse HEAD` \
        --build-arg CREATED=`date -u +%Y-%m-%dT%H:%M:%SZ` \
        -t plexeditions:dev .
