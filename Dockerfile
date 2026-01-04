# -------- Build stage --------
FROM rust:1-bullseye as builder

WORKDIR /app

# Create a dummy crate to prime dependency cache
RUN cargo new iploc
WORKDIR /app/iploc

# Copy manifests first to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Pre-build dependencies (will fail, but pulls deps into cache)
RUN cargo build --release || true

# Now copy the full source
COPY src ./src
COPY README.md ./README.md
COPY tests ./tests

# Build release binary
RUN cargo build --release

# -------- Runtime stage --------
FROM debian:bullseye-slim AS runtime

WORKDIR /app

# Install CA certs so reqwest/rustls can talk to https://api.ip2location.io
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /app/iploc/target/release/iploc /app/iploc

# Expose the Axum port
EXPOSE 3000

ENV RUST_LOG=info

CMD ["/app/iploc"]
