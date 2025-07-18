### Builder stage
# Using 'rust:slim' which defaults to the latest stable slim image.
# For truly reproducible builds, consider pinning to a specific version,
# e.g., 'rust:1.78-slim' (or whatever the current stable is at that time).
FROM rust:slim AS builder

# Set the working directory inside the container for the backend source
WORKDIR /usr/src/backend

# Accept build-time environment variables for all config
ARG DATABASE_URL
ARG FRONTEND_REQUEST_URL
ARG VITE_API_URL
ARG PORT
ARG HOST
ARG JWT_SECRET
ARG SMTP_USERNAME
ARG SMTP_PASSWORD
ARG SMTP_PORT
ARG SMTP_FROM_EMAIL
ARG SMTP_FROM_NAME
ARG SMTP_HOST
ARG UPSTASH_REDIS_REST_URL
ARG UPSTASH_REDIS_REST_TOKEN
ARG ADMIN_SECRET_KEY

ENV DATABASE_URL=${DATABASE_URL}
ENV FRONTEND_REQUEST_URL=${FRONTEND_REQUEST_URL}
ENV VITE_API_URL=${VITE_API_URL}
ENV PORT=${PORT}
ENV HOST=${HOST}
ENV JWT_SECRET=${JWT_SECRET}
ENV SMTP_USERNAME=${SMTP_USERNAME}
ENV SMTP_PASSWORD=${SMTP_PASSWORD}
ENV SMTP_PORT=${SMTP_PORT}
ENV SMTP_FROM_EMAIL=${SMTP_FROM_EMAIL}
ENV SMTP_FROM_NAME=${SMTP_FROM_NAME}
ENV SMTP_HOST=${SMTP_HOST}
ENV UPSTASH_REDIS_REST_URL=${UPSTASH_REDIS_REST_URL}
ENV UPSTASH_REDIS_REST_TOKEN=${UPSTASH_REDIS_REST_TOKEN}
ENV ADMIN_SECRET_KEY=${ADMIN_SECRET_KEY}

# Install build dependencies in a single RUN command to create fewer layers
# and ensure apt cache is cleaned immediately.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        curl \
        binutils-dev \
    && rm -rf /var/lib/apt/lists/*

# IMPORTANT: Update rustup and cargo to ensure compatibility with your Cargo.lock file.
# This fixes the "lock file version" error and ensures latest tools.
RUN rustup update stable && rustup default stable

# --- Start of Rust dependency caching strategy ---
# Copy only Cargo.toml and Cargo.lock first to leverage Docker's build cache.
# This layer only changes if your project dependencies change.
# We copy `backend/Cargo.toml` and `backend/Cargo.lock` from the host into `./` (WORKDIR) in the container.
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create a dummy src/main.rs and perform a dummy build.
# This pre-caches all your Rust dependencies. If only your source code changes,
# this layer will be pulled from cache, speeding up subsequent builds.
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && cargo build --release || true

# Clean up the dummy source and target directories after caching dependencies
# to avoid conflicts with the actual code copy later.
RUN rm -rf src target/release/backend target/debug/backend
# --- End of Rust dependency caching strategy ---

# Copy the migrations directory (important for `sqlx-cli` if run during build)
COPY backend/migrations ./migrations

# Copy the entire backend source code into the working directory.
COPY backend .

# Build the actual application. This will be faster due to cached dependencies.
RUN cargo build --release

# --- OPTIMIZATION FOR BINARY SIZE ---
# Explicitly strip all debug symbols and unnecessary information from the binary.
# This uses the system's `strip` utility. It should be run AFTER `cargo build --release`.
RUN strip /usr/src/backend/target/release/backend
# --- END OPTIMIZATION ---


### Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies in a single RUN command
# ca-certificates is good for validating HTTPS connections. libssl3 for OpenSSL.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security best practices
RUN useradd -m -U -s /bin/false app

# Set the working directory for the runtime environment
WORKDIR /app

# Copy the compiled and stripped binary from the builder stage
# Ensure the path matches the builder's WORKDIR and binary output.
COPY --from=builder /usr/src/backend/target/release/backend /app/backend

# Copy migrations directory for runtime execution (e.g., if you run migrations at startup)
# Ensure this path is correct if your Rust app needs to access `migrations` at runtime.
COPY --from=builder /usr/src/backend/migrations /app/migrations

# # Define runtime arguments (these should ideally be passed at `docker run` or in `docker-compose.yml`)
ARG DATABASE_URL
ARG FRONTEND_REQUEST_URL
ARG VITE_API_URL
ARG PORT
ARG HOST
ARG JWT_SECRET
ARG SMTP_USERNAME
ARG SMTP_PASSWORD
ARG SMTP_PORT
ARG SMTP_FROM_EMAIL
ARG SMTP_FROM_NAME
ARG SMTP_HOST
ARG UPSTASH_REDIS_REST_URL
ARG UPSTASH_REDIS_REST_TOKEN
ARG ADMIN_SECRET_KEY

# # Set runtime environment variables using the ARG values.
# # These will be the actual environment variables available to your running application.
ENV DATABASE_URL=${DATABASE_URL}
ENV FRONTEND_REQUEST_URL=${FRONTEND_REQUEST_URL}
ENV VITE_API_URL=${VITE_API_URL}
ENV PORT=${PORT}
ENV HOST=${HOST}
ENV JWT_SECRET=${JWT_SECRET}
ENV SMTP_USERNAME=${SMTP_USERNAME}
ENV SMTP_PASSWORD=${SMTP_PASSWORD}
ENV SMTP_PORT=${SMTP_PORT}
ENV SMTP_FROM_EMAIL=${SMTP_FROM_EMAIL}
ENV SMTP_FROM_NAME=${SMTP_FROM_NAME}
ENV SMTP_HOST=${SMTP_HOST}
ENV UPSTASH_REDIS_REST_URL=${UPSTASH_REDIS_REST_URL}
ENV UPSTASH_REDIS_REST_TOKEN=${UPSTASH_REDIS_REST_TOKEN}
ENV ADMIN_SECRET_KEY=${ADMIN_SECRET_KEY}

# Change ownership of the /app directory to the non-root user
RUN chown -R app:app /app

# Switch to the non-root user for running the application
USER app

# Expose the port your application listens on
EXPOSE 8081

# Define the command to run your application
CMD ["./backend"]