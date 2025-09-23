# Multi-stage build for Rust 1.90 Workflow System
FROM rust:1.90-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/
COPY examples/ ./examples/
COPY benches/ ./benches/
COPY tests/ ./tests/

# Build the application
RUN cargo build --release --features rust190

# Build examples
RUN cargo build --examples --release --features rust190

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r workflow && useradd -r -g workflow workflow

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/workflow /app/workflow
COPY --from=builder /app/target/release/examples/ /app/examples/

# Change ownership to non-root user
RUN chown -R workflow:workflow /app

# Switch to non-root user
USER workflow

# Expose port (if needed)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["./workflow"]
