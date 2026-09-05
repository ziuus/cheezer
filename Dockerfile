# Stage 1 (Builder)
FROM gcr.io/distroless/cc-debian12 AS builder
WORKDIR /app
COPY cheezer-bin ./cheezer-core

# Stage 2 (Runtime)
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/cheezer-core ./cheezer-core
EXPOSE 9090 9000
CMD ["./cheezer-core", "--role=primary"]
