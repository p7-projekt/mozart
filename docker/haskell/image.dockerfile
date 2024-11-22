FROM --platform=linux/amd64 rust:1.82 AS build
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /build
COPY . /build
RUN cargo build --locked --release --target=x86_64-unknown-linux-musl --features haskell

FROM --platform=linux/amd64 alpine:3.20
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/mozart /bin/mozart
RUN apk add --no-cache \
    ghc=9.8.2-r1 \
    musl-dev \
    gcc \
    libc-dev \
    shadow
RUN useradd -M -N restricted # -M means no home folder, -N means no user group

ENV PATH="$PATH:/usr/bin/ghc"
EXPOSE 8080
CMD ["/bin/mozart"]
