# build
FROM rust:1.94-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

COPY src ./src
RUN cargo build --release

# runtime
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rhizome /usr/local/bin/rhizome

EXPOSE 3000

CMD ["rhizome"]