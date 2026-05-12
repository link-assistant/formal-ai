FROM rust:1.82-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/formal-ai /usr/local/bin/formal-ai

EXPOSE 8080
ENTRYPOINT ["formal-ai"]
CMD ["serve", "--host", "0.0.0.0", "--port", "8080"]
