FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -U -s /bin/false app

WORKDIR /app

# Copy the pre-built binary from the host
COPY backend/package/backend /app/backend

# Copy migrations if needed
COPY backend/migrations /app/migrations

# Set environment variables (as before)
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

RUN chown -R app:app /app
USER app

EXPOSE 8081

CMD ["./backend"]