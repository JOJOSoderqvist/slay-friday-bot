# Stage 1: Builder
FROM rust:latest AS builder

# Install OpenSSL development libraries
RUN apt-get update && apt-get install -y libssl-dev pkg-config build-essential

WORKDIR /app

# Copy your Rust project files
COPY . .

# Build your Rust application (adjust as needed for your specific project)
RUN cargo build --release

# Stage 2: Runtime
FROM debian:stable-slim

# Install OpenSSL runtime libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

WORKDIR /app

# Copy the compiled executable from the builder stage
COPY --from=builder /app/target/release/slayFridayBot .

# Set the entrypoint for your application
CMD ["./slayFridayBot"]