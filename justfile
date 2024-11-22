PLATFORM := "--target=x86_64-unknown-linux-musl"

# Lists justfile recipes.
default:
    just -l

# Builds and runs the mozart container.
run LANGUAGE:
    @just build {{LANGUAGE}}
    docker run -p 8080:8080 -d mozart-{{LANGUAGE}}

# Run language specific test cases in Docker image.
test LANGUAGE:
    docker build . -t mozart-{{LANGUAGE}}-test -f docker/{{LANGUAGE}}/test.dockerfile
    docker run mozart-{{LANGUAGE}}-test

# Build a language specific mozart image.
build LANGUAGE:
    docker build . -t mozart-{{LANGUAGE}} -f docker/{{LANGUAGE}}/image.dockerfile

# Compile and open documentation.
doc LANGUAGE="default":
    cargo doc --open --document-private-items --features {{LANGUAGE}}
