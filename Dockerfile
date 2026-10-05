# Stage 1: Base with cargo-chef
FROM lukemathwalker/cargo-chef:0.1.78-rust-1.99.0-trixie AS chef
WORKDIR /app

# Stage 2: Planner (generates recipe.json for dependency caching)
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder (cooks dependencies, then compiles binaries)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin theater-api --bin queue_worker --bin seed

# Stage 4: Minimal runtime
FROM debian:trixie-slim AS runtime
WORKDIR /app

# Install runtime dependencies (OpenSSL 3 runtime, CA certs, timezone data)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    openssl \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binaries to PATH
COPY --from=builder /app/target/release/theater-api /usr/local/bin/theater-api
COPY --from=builder /app/target/release/queue_worker /usr/local/bin/queue_worker
COPY --from=builder /app/target/release/seed /usr/local/bin/seed

ENV TZ=Asia/Bangkok
ENV PORT=3000
EXPOSE 3000

CMD ["theater-api"]
