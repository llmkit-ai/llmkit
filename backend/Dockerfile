# Builder stage
FROM rust:1.85 as builder

WORKDIR /app

# Install sqlx-cli for migrations and preparation
RUN cargo install sqlx-cli --no-default-features --features sqlite

# Copy Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy migrations and source code
COPY migrations /app/migrations
COPY . .
RUN mkdir /app/data
RUN cargo install sqlx-cli
RUN sqlx database create && sqlx migrate run
#RUN mv ./llmkit.db /app/data/
RUN cargo build --bin backend --release

# Create temporary database, run migrations, and prepare offline data
RUN sqlx database create --database-url sqlite:/tmp/build.db
RUN sqlx migrate run --database-url sqlite:/tmp/build.db
RUN cargo sqlx prepare --database-url sqlite:/tmp/build.db

# Force SQLx offline mode and build the release binary
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary and migrations from the builder
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/data/llmkit.db /app/data/
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/.env /app/

# Copy and set up the entrypoint script (assuming it exists)
COPY --from=builder /app/docker-entrypoint.sh /app/
RUN chmod +x /app/docker-entrypoint.sh

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/llmkit.db

# Expose the port
EXPOSE 8000

# Run the app
ENTRYPOINT ["/app/docker-entrypoint.sh"]
