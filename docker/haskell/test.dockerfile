FROM --platform=linux/amd64 rust:alpine3.20

RUN apk add --no-cache \
    ghc=9.8.2-r1 \
    musl-dev \
    shadow
RUN mkdir /mozart
RUN useradd -M -N restricted # -M means no home folder, -N means no user group
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /test
COPY . .
ENV PATH="$PATH:/usr/bin/ghc"
CMD ["cargo", "test", "--target=x86_64-unknown-linux-musl", "--features", "haskell", "--features", "ci"]
