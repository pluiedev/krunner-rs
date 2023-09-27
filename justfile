alias d := doc
alias e := example

# Runs an example with all features.
example name:
    cargo r --example {{name}} --all-features

# Builds the documentation for this crate, as how it would appear on docs.rs.
doc *args:
   RUSTDOCFLAGS='--cfg docs_rs' cargo doc --all-features {{args}}
