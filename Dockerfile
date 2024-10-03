FROM rust:1.81 AS build
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /build
COPY . /build
RUN cargo build --locked --release --target=x86_64-unknown-linux-musl

FROM alpine:3.20
ENV PATH="$PATH:/usr/bin/ghc"
COPY /target/x86_64-unknown-linux-musl/release/mozart /bin/mozart
RUN apk add --no-cache "ghc=9.8.2-r1"
EXPOSE 8080
CMD ["/bin/mozart"]