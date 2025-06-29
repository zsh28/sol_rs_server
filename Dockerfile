# Stage 1: Build with musl for static linking
FROM rustlang/rust:nightly AS builder

# Install musl and build tools
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev

WORKDIR /app
COPY . .

# Build a statically linked binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Ubuntu-based minimal runtime image
FROM ubuntu:24.04.1

# Optional: reduce size
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m appuser
USER appuser

# Copy the statically linked binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/sol_rs_server /usr/local/bin/app

EXPOSE 3000
CMD ["app"]
