# Use official Rust image as a base
FROM rust:latest AS chef
RUN cargo install cargo-chef --locked --version 0.1.68

# Set the working directory
WORKDIR /app

# Copy the project files to the working directory for chef
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Install system dependencies (needed for Postgres)
RUN apt-get update && apt-get install -y \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

# Build the project in release mode
RUN cargo build --release

# Stage 2 - Runtime image
FROM debian:bookworm-slim AS runner

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    openssl \
    postgresql-client \ 
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/static /app/static
COPY --from=builder /app/target/release/todoapp /app/todoapp

# Copy the SQL init script and entrypoint
COPY ./init.sql /docker-entrypoint-initdb.d/
COPY ./entrypoint.sh /app/entrypoint.sh

# Make entrypoint executable
RUN chmod +x /app/entrypoint.sh

# Expose the port for the application
EXPOSE 8080

# Set the entrypoint script
ENTRYPOINT ["/app/entrypoint.sh"]
