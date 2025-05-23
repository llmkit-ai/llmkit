# Builder stage
FROM rust:1.85 AS builder

WORKDIR /app

# Install sqlx-cli using a faster method - use cargo-binstall for pre-compiled binaries
RUN cargo install cargo-binstall
RUN cargo binstall sqlx-cli --no-default-features --features sqlite --no-confirm

# Copy only Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create src directory and dummy main.rs
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached if Cargo.toml/Cargo.lock don't change)
RUN cargo build --release

# Remove dummy files
RUN rm -rf src target/release/deps/backend*

# Now copy the actual source code
COPY src ./src
COPY migrations ./migrations

# Create temporary database, run migrations, and prepare offline data
RUN sqlx database create --database-url sqlite:/tmp/build.db
RUN sqlx migrate run --database-url sqlite:/tmp/build.db  
RUN cargo sqlx prepare --database-url sqlite:/tmp/build.db

# Build the actual application (dependencies are already compiled and cached)
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Final stage with minimal runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies in a single layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Copy the built binary and migrations from builder stage
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/migrations /app/migrations

# Create non-root user for security
RUN useradd -m -u 1001 appuser && chown -R appuser:appuser /app
USER appuser

# Expose the port
EXPOSE 8000

# Run the app
CMD ["./backend"]