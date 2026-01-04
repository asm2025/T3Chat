# Multi-stage Dockerfile for T3Chat React Frontend
# Builds the frontend and prepares it for serving from backend wwwroot

# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Install pnpm
RUN npm install -g pnpm

# Copy package files
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
# The build output will be in dist/
RUN pnpm run build

# Output stage - just copy the built files
FROM scratch AS export
COPY --from=builder /app/dist /

