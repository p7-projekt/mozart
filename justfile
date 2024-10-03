default:
    just -l

run:
    cargo build --locked --release --target=x86_64-unknown-linux-musl
    ./target/x86_64-unknown-linux-musl/release/mozart

test:
    cargo test -q

dbuild:
    docker build -t mozart

drun:
    docker run -p 8080:8080 -d mozart
