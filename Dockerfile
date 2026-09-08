# mytest — integration test suite
# No database needed, no web server — pure CLI tool

FROM rust:1.82-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mytest /usr/local/bin/mytest
ENTRYPOINT ["mytest"]
