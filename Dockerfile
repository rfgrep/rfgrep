## Multi-stage Dockerfile for rfgrep
# Build stage
FROM rust:1.91-slim AS builder
RUN apt-get update && apt-get upgrade -y && apt-get install -y --no-install-recommends ca-certificates build-essential pkg-config libssl-dev git && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/rfgrep
COPY . .
RUN useradd --no-create-home --shell /bin/false builder
RUN cargo build --release --locked --bin rfgrep

# Final stage
FROM gcr.io/distroless/cc
COPY --from=builder /usr/src/rfgrep/target/release/rfgrep /usr/local/bin/rfgrep
USER nonroot
WORKDIR /home/nonroot
ENTRYPOINT ["/usr/local/bin/rfgrep"]
CMD ["--help"]
