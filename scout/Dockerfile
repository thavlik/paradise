FROM thavlik/rust-musl-builder:latest AS builder
COPY types types
RUN cargo new scout
COPY scout/Cargo.toml scout/Cargo.toml
COPY Cargo.lock scout/Cargo.lock

WORKDIR /home/rust/scout
RUN cargo build --release --target=x86_64-unknown-linux-musl \
    && rm src/*.rs \
    && rm ./target/x86_64-unknown-linux-musl/release/deps/scout*
COPY scout/src src

RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:3.11.5
COPY --from=builder /home/rust/scout/target/x86_64-unknown-linux-musl/release/scout /usr/local/bin/scout
CMD ["scout"]
