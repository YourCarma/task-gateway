FROM rust:1.94 AS chef

WORKDIR /app

RUN cargo install cargo-chef --locked


# Planner layer with cargo-chef cli tool and projects sources to create recipe.json
FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json


# Builder layer with build project binaries based on previous planner layer
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY . .

RUN cargo build --release --locked --bins


# Target layer based on official ubuntu image with neccessary binaries and data to run.
FROM ubuntu:24.04

RUN apt-get update \
 && apt-get install -y --no-install-recommends openssl ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --uid 10001 --create-home appuser

WORKDIR /app

COPY ./config /app/config
COPY --from=builder /app/target/release/run_server .

USER appuser

ENTRYPOINT ["/app/run_server"]

EXPOSE 10010
