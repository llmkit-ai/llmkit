# Use slim image for faster downloads
FROM rust:1.85-slim AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

WORKDIR /app

# Install required build dependencies and sccache
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli for migrations and cargo-watch for development hot reload (cache this layer)
RUN cargo install sqlx-cli --no-default-features --features sqlite && \
    cargo install cargo-watch

# Copy recipe and cook dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/var/cache/sccache \
    apt-get update && apt-get install -y sccache && \
    export RUSTC_WRAPPER=sccache && \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY src src/
COPY migrations migrations/
COPY Cargo.toml Cargo.lock ./

# Create temporary database and prepare SQLx
RUN sqlx database create --database-url sqlite:/tmp/build.db && \
    sqlx migrate run --database-url sqlite:/tmp/build.db && \
    cargo sqlx prepare --database-url sqlite:/tmp/build.db

# Build with SQLx offline mode and sccache
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/var/cache/sccache \
    export RUSTC_WRAPPER=sccache && \
    cargo build --release

# Runtime stage with minimal image
FROM debian:bookworm-slim

WORKDIR /app

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy only the necessary files
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8000
CMD ["./backend"]