FROM rust:1.87.0-bullseye AS base

WORKDIR /build

COPY crates ./crates
COPY Cargo.toml Cargo.lock ./
COPY .sqlx .sqlx

RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/build/target/deps \
  --mount=type=cache,target=/build/target/build \
  cargo build --release

FROM rockylinux/rockylinux:9-ubi-micro AS run

COPY --from=base /build/target/release/blogi /usr/local/bin/blogi

ENTRYPOINT [ "/usr/local/bin/blogi" ]
