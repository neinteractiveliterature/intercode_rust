FROM rust:1-slim-bullseye AS base
WORKDIR /usr/src/intercode_rust
RUN cargo install cargo-chef

FROM base as plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS build
COPY --from=plan /usr/src/intercode_rust/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build -r

FROM debian:bullseye-slim AS release

COPY --from=build /usr/src/intercode_rust/target/release/intercode_rust /usr/local/bin
CMD intercode_rust serve
