PLATFORM := "--target=x86_64-unknown-linux-musl"

default:
    just -l

# Build and execute crate
dev LANGUAGE:
    MOZART_LOG=TRACE cargo build --locked --release {{PLATFORM}} --features {{LANGUAGE}}
    ./target/x86_64-unknown-linux-musl/release/mozart

# Build and execute crate
run LANGUAGE:
    cargo build --locked --release {{PLATFORM}} --features {{LANGUAGE}}
    ./target/x86_64-unknown-linux-musl/release/mozart

# Run test cases
test LANGUAGE:
    cargo test -q {{PLATFORM}} --features {{LANGUAGE}}

# Build the mozart image
dbuild LANGUAGE:
    docker build . -t {{LANGUAGE}} -f docker/{{LANGUAGE}}

# Runs the mozart container
drun LANGUAGE:
    docker run -p 8080:8080 -d {{LANGUAGE}}
