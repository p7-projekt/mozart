default:
    just -l

run:
    cargo build --locked --release --target=x86_64-unknown-linux-musl
    ./target/x86_64-unknown-linux-musl/release/mozart

test:
    cargo test -q
