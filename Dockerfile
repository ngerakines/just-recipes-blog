FROM rust:1-bookworm as builder
ENV HOME=/root
WORKDIR /app/
COPY Cargo.toml Cargo.lock build.rs /app/
COPY src/ /app/src/
RUN find /app/
ARG CARGO_ARGS=""
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/root/app/target cargo build --release --color never ${CARGO_ARGS}

FROM debian:bookworm-slim
ENV RUST_LOG="warning"
COPY --from=builder /app/target/release/jrb /usr/local/bin/jrb
CMD ["sh", "-c", "/usr/local/bin/jrb"]
