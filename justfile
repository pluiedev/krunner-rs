alias b := build
alias c := check
alias d := doc
alias e := example

build:
    cargo build --all-features

check:
    cargo clippy --all-features

# Builds the documentation for this crate, as how it would appear on docs.rs.
doc *args:
   RUSTDOCFLAGS='--cfg docs_rs' cargo doc --all-features {{args}}

# Runs an example with all features.
example name:
    cargo run --example {{name}} --all-features

test:
    cargo test --all-features
