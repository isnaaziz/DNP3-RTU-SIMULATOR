# syntax=docker/dockerfile:1

FROM rust:1.91-slim-bookworm AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src target/release/deps/simulator_rtu_dnp3* target/release/simulator-rtu-dnp3

COPY src ./src
COPY static ./static
RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /app simulator \
    && mkdir -p /data \
    && chown simulator /data

ENV TZ=Asia/Jakarta
WORKDIR /app
COPY --from=builder /build/target/release/simulator-rtu-dnp3 /app/simulator-rtu-dnp3

USER simulator
VOLUME ["/data"]
EXPOSE 8080 20000

HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/api/state > /dev/null || exit 1

ENTRYPOINT ["/app/simulator-rtu-dnp3"]
CMD ["--data", "/data/simulator-data.json", "--http", "0.0.0.0:8080"]
