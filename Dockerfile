FROM rust:1.91 AS chef

WORKDIR /app

RUN cargo install cargo-chef


# Planner layer with cargo-chef cli tool and projects sources to create recipe.json
FROM chef AS planner

RUN apt update && apt install -y libssl-dev

COPY . .

RUN cargo chef prepare --recipe-path recipe.json


# Builder layer with build project binaries based on previous planner layer
FROM chef AS builder

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo install ${FEATURES} --bins --path .


# Target layer based on tiny official ubuntu image with neccessary binaries and data to run.
FROM ubuntu:24.04

# Добавьте эти строки перед установкой пакетов
RUN sysctl -w net.ipv6.conf.all.disable_ipv6=1 \
 && sysctl -w net.ipv6.conf.default.disable_ipv6=1
 
RUN apt-get update && apt install -y openssl ca-certificates curl
WORKDIR /app

COPY ./config /app/config
COPY --from=builder /app/target/release/run_server .

# Execute to initliaze elasticsearch environment

ENTRYPOINT ["/app/run_server"]

EXPOSE 10001
