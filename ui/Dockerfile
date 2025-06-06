# Build stage
FROM oven/bun:latest AS builder

WORKDIR /app

# Copy package files for dependency caching
COPY package.json bun.lock* ./

# Build argument for lock file hash
ARG BUN_LOCK_HASH
LABEL bun.lock.hash=$BUN_LOCK_HASH

# Install dependencies with cache mount
RUN --mount=type=cache,target=/root/.bun/install/cache \
    bun install

# Copy source code
COPY . .

# Build arguments
ARG API_BASE_URL
ENV API_BASE_URL=$API_BASE_URL

# Build the application
RUN bun run build

# Production stage with minimal image
FROM node:lts-alpine

WORKDIR /app

# Copy only the built application
COPY --from=builder /app/.output ./.output
COPY --from=builder /app/package.json ./package.json

# Install only production dependencies if needed
RUN npm ci --only=production && npm cache clean --force

EXPOSE 3000

CMD ["node", ".output/server/index.mjs"]
