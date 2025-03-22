# ========= Build stage =========
FROM rust:1.81 as builder

WORKDIR /app

# 1) Copy just your cargo files and create a placeholder for dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs

# 2) Pre-build to cache dependencies
RUN cargo build --bin backend --release
RUN rm -rf src  # remove the placeholder

# 3) Copy the actual source code and build
COPY . .
RUN mkdir /app/data
RUN cargo install sqlx-cli
RUN sqlx database create && sqlx migrate run
#RUN mv ./llmkit.db /app/data/
RUN cargo build --bin backend --release

# ========= Final minimal stage =========
FROM debian:bookworm-slim

WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/data/llmkit.db /app/data/
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/.env /app/

EXPOSE 8000

CMD ["./backend"]
