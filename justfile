default:
    just -l

# Build and execute crate
run:
    cargo build --locked --release --target=x86_64-unknown-linux-musl
    ./target/x86_64-unknown-linux-musl/release/mozart

# Run test cases
test:
    cargo test -q --target=x86_64-unknown-linux-musl

# Build the mozart image
dbuild TARGET:
    docker build . -t {{TARGET}} -f docker/{{TARGET}}

# Runs the mozart container
drun TARGET:
    docker run -p 8080:8080 -d {{TARGET}}
