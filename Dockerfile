FROM ubuntu:24.04
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY cheezer-bin ./cheezer-core
EXPOSE 9090 9000
CMD ["./cheezer-core", "--role=primary"]
