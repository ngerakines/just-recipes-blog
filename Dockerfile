FROM rust:1.57 as builder
WORKDIR /usr/src/jrb
COPY . .
RUN cargo install --path .
RUN find /usr/src/jrb

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/jrb /usr/local/bin/jrb
CMD ["jrb"]
