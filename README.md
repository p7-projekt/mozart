Mozart is a test composer and runner. It is designed to support any language that has a language handler implemented.

It checks a submitted program against a set of test cases, and reports back the result of this check.

Below you can find a link to each of the published Mozart containers.

<details>
  <summary>
    DockerHub links
  </summary>
    <ul>
      <li>
        <a href="https://hub.docker.com/repository/docker/p7project/mozart-python/general">Python</a>
      </li>
      <li>
        <a href="https://hub.docker.com/repository/docker/p7project/mozart-haskell/general">Haskell</a>
      </li>
    </ul>
</details>

# Why Rust?

Mozart is intended to be deployed inside a linux container and to be as small as possible, so as to not take ressources from the work it is doing.

As such, a couple of requirements were specified:

- **minimal overhead**: because mozart deals with compiling and running programs, with mozart just being an interface, it should be minimal and not take up a lot of space or ressources
- **compile to statically linked binary**: for ease of deployment, it is nice to compile to a statically linked binary and not thing about it further

Because of the first requirement, languages that require an interpreter or a large runtime were discarded.

And due to the second requirement the option was between Golang and Rust, although other languages of course fulfill both requirements

Golang was considered initially, but was later replaced by Rust due to developer experience and comfort.

Rust also allowed the language support to be enabled as a compile time feature, thereby ensuring that exactly one language is enabled for mozart to compile.
Furthermore, it also means that even if hundreds of languages are supported, only one is enabled and part of the binary, thereby limiting the binary size.

It is also important to note that performance was not a requirement or consideration, as the bottleneck is compiling and executing the submitted program, not the mozart code.

# Requirements

To run mozart you require:

- the rust compiler (with the `x86_64-unknown-linux-musl` target installed for statically linked binary)
- the compiler/interpreter of the language you wish to run

# Run

To run mozart you can use the following command:

```
cargo run --locked --release --target=x86_64-unknown-linux-musl --features {{LANGUAGE}}
```

Here, the `{{LANGUAGE}}` refers to the language instance you want to enable, for example `haskell`.

Depending on how you installed your language compiler/interpreter, you may need to run mozart as a super user, to access its dependencies.

During development you can change the log level via the `MOZART_LOG` environment variable - it is set to `info` by default.

# Adding a Language

Mozart is designed to relatively easily support a new language. You need to:

- add a language feature to the `Cargo.toml` for the language you wish to support
- create a new module inside `src/runner` named after the language
- implement the `LanguageHandler` trait for your language handler
- add you language handler as a conditional field (based on language feature) to the `TestRunner` in `src/runner/mod.rs`

You can look at the existing supported languages for an idea of how it should look.
