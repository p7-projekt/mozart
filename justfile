PLATFORM := "--target=x86_64-unknown-linux-musl"

default:
    just -l

# Build and execute crate
dev LANGUAGE:
    MOZART_LOG=TRACE cargo run --locked --release {{PLATFORM}} --features {{LANGUAGE}}

# Build and execute crate
run LANGUAGE:
    cargo run --locked --release {{PLATFORM}} --features {{LANGUAGE}}

# Run test cases
test LANGUAGE:
    cargo test {{PLATFORM}} --features {{LANGUAGE}}

# Build the mozart image
dbuild LANGUAGE:
    docker build . -t {{LANGUAGE}} -f docker/{{LANGUAGE}}

# Runs the mozart container
drun LANGUAGE:
    docker run -p 8080:8080 -d {{LANGUAGE}}

# Compile and open documentation.
doc LANGUAGE="default":
    cargo doc --open --document-private-items --features {{LANGUAGE}}
