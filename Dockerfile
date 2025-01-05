# Use the official Rust image as a builder
FROM rust:1.74-slim-bookworm as builder

# Install required dependencies for building
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libopus-dev make gcc g++ && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the application with release profile
RUN cargo build --release

# Create a new stage with a minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl3 libopus0 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/taroj1205-discord-bot /usr/local/bin/discord-bot

# Create a directory for the database
RUN mkdir -p /usr/local/share/discord-bot
WORKDIR /usr/local/share/discord-bot

# Copy the .env file if it exists
COPY --from=builder /usr/src/app/.env ./.env

# Run the bot
CMD ["discord-bot"]
