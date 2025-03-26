# Builder stage
FROM rust:1.85 AS builder

WORKDIR /app

# Install sqlx-cli for migrations and preparation
RUN cargo install sqlx-cli --no-default-features --features sqlite

# Copy Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy migrations and source code
COPY . .

# Create temporary database, run migrations, and prepare offline data
RUN sqlx database create --database-url sqlite:/tmp/build.db
RUN sqlx migrate run --database-url sqlite:/tmp/build.db
#RUN cargo sqlx prepare --database-url sqlite:/tmp/build.db

# Force SQLx offline mode and build the release binary
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary and migrations from the builder
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/migrations /app/migrations

# Expose the port
EXPOSE 8000

# Run the app
CMD ["./backend"]