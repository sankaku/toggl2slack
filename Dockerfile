FROM ekidd/rust-musl-builder:1.47.0 AS builder

COPY ./Cargo.toml /home/rust/src/Cargo.toml
COPY ./Cargo.lock /home/rust/src/Cargo.lock
COPY ./src /home/rust/src/src

RUN cargo build --release

FROM alpine:3.12.1
# To use GNU date command in CDK setting
RUN apk add --update-cache coreutils
WORKDIR /app
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/toggl2slack /app/
ENV TZ Asia/Tokyo
CMD ["/app/toggl2slack"]

