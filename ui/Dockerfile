FROM oven/bun:latest AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lock ./

# Install dependencies
RUN bun install

# Copy the rest of the application
COPY . .

# Build the application
RUN bun run build

FROM node:lts-slim

WORKDIR /app

COPY --from=builder /app .

ENV HOST 0.0.0.0
EXPOSE 3000

CMD ["node", ".output/server/index.mjs"]
