FROM --platform=linux/amd64 rust:alpine3.20

RUN apk add --no-cache \
    python3=3.12.7-r0 \
    musl-dev \
    shadow
RUN mkdir /mozart
RUN useradd -M -N restricted # -M means no home folder, -N means no user group
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /test
COPY . .
ENV PATH="$PATH:/usr/bin/python"
CMD ["cargo", "test", "--target=x86_64-unknown-linux-musl", "--features", "python"]
