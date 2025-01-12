# Build stage
FROM rust:1.72 as builder

WORKDIR /app
COPY . .

# Install dependencies and build
RUN cargo build --release

# Runtime stage
FROM debian:stable-slim

# Install SSL certificates and other runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config files
COPY --from=builder /app/target/release/degressly-core-rust /usr/local/bin/
COPY --from=builder /app/config /etc/degressly-core/config

# Set environment variables
ENV RUN_MODE=production
ENV APP_CONFIG_DIR=/etc/degressly-core/config

# Expose the service port
EXPOSE 8000

# Run the service
CMD ["degressly-core-rust"]
