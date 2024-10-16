PLATFORM := "--target=x86_64-unknown-linux-musl"

default:
    just -l

# Build and execute crate
dev TARGET:
    MOZART_LOG=TRACE cargo build --locked --release {{PLATFORM}} --features {{TARGET}}
    ./target/x86_64-unknown-linux-musl/release/mozart

# Build and execute crate
run TARGET:
    cargo build --locked --release {{PLATFORM}} --features {{TARGET}}
    ./target/x86_64-unknown-linux-musl/release/mozart

# Run test cases
test TARGET:
    cargo test -q {{PLATFORM}} --features {{TARGET}}

# Build the mozart image
dbuild TARGET:
    docker build . -t {{TARGET}} -f docker/{{TARGET}}

# Runs the mozart container
drun TARGET:
    docker run -p 8080:8080 -d {{TARGET}}
