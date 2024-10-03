default:
    just -l

# Build and execute crate
run:
    cargo build --locked --release --target=x86_64-unknown-linux-musl
    ./target/x86_64-unknown-linux-musl/release/mozart

# Run test cases
test:
    cargo test -q

# Build the mozart image
dbuild:
    docker build -t mozart .

# Runs the mozart container
drun:
    docker run -p 8080:8080 -d mozart
