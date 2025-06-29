# Stage 1: Build with nightly
FROM rustlang/rust:nightly AS builder

WORKDIR /app
COPY . .

RUN rustup component add rustfmt
RUN cargo build --release

# Stage 2: Create minimal runtime image
FROM debian:bullseye-slim
RUN adduser --disabled-password appuser
USER appuser

COPY --from=builder /app/target/release/sol_rs_server /usr/local/bin/app
EXPOSE 3000
CMD ["app"]
