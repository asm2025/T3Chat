# Multi-stage Dockerfile for T3Chat Rust Backend
# Based on LibreChat Dockerfile patterns
# Build context should be the project root directory

# Frontend build stage
FROM node:20-alpine AS frontend-builder

WORKDIR /app/frontend

# Install pnpm
RUN npm install -g pnpm

# Copy frontend package files
COPY client/package.json client/pnpm-lock.yaml client/pnpm-workspace.yaml ./
COPY client/tsconfig*.json ./
COPY client/vite.config.ts ./
COPY client/index.html ./
COPY client/public ./public
COPY client/src ./src

# Install dependencies and build
RUN pnpm install --frozen-lockfile && \
    pnpm run build

# Backend build stage
FROM rust:1.82-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    postgresql-dev \
    pkgconfig \
    openssl-dev

WORKDIR /app

# Copy dependency files first for better caching
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/diesel.toml ./

# Copy source code
COPY server/src ./src
COPY server/migrations ./migrations

# Copy frontend build to wwwroot (before building backend)
COPY --from=frontend-builder /app/frontend/dist ./wwwroot

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    postgresql-client \
    libpq \
    openssl \
    wget

# Create app user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/t3chtsrv /app/t3chtsrv

# Copy frontend build (wwwroot) from builder
COPY --from=builder /app/wwwroot /app/wwwroot

# Copy configuration files
COPY server/t3chat.yaml /app/t3chat.yaml
COPY server/t3chat.example.yaml /app/t3chat.example.yaml

# Ensure proper permissions
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port (default 3000, can be overridden)
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:${PORT:-3000}/health || exit 1

# Run the application
CMD ["/app/t3chtsrv"]

