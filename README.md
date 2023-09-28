# `krunner-rs`

A crate for writing [KRunner](https://userbase.kde.org/Plasma/Krunner) plugins in Rust, easily.

Check out the [examples](https://github.com/pluiedev/krunner-rs) for more!

# Feature flags

`krunner-rs` uses some feature flags to enable optional features.
Here's all that's currently supported:

- `tokio`: Enables [asynchronous runners](AsyncRunner) with the [Tokio](::tokio) runtime.
- `derive`: Enables generating [`Action`]s with a [derive macro](derive@Action).
