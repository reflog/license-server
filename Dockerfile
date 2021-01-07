# 1: Build the exe
FROM rust:1.42 as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN USER=root cargo new myprogram
WORKDIR /usr/src/myprogram
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
ENV PORT=8000
ENV LICENSE_API_KEY=
ENV HMAC_SECRET=
COPY --from=builder /usr/src/myprogram/target/x86_64-unknown-linux-musl/release/license-server .
USER 1000
CMD ["./license-server"]
