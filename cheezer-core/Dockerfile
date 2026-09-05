FROM rust:slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/cheezer-core /app/cheezer-core
CMD ["/app/cheezer-core"]
