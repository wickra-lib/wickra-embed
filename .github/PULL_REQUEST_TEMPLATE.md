<!-- Keep it short. One logical change per PR. -->

## What

<!-- What does this change and why? -->

## Checklist

- [ ] `cargo fmt --all` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` are clean
- [ ] The no_std core builds on the bare-metal targets (`cargo build -p embed-core --no-default-features --target thumbv7em-none-eabihf` and `thumbv6m-none-eabi`)
- [ ] `cargo test -p embed-core --all-features` passes, including the byte-parity tests against `wickra-core`
- [ ] `cargo deny check` is clean
- [ ] Determinism preserved (fixed f64 operation order; no alloc, HashMap, time or RNG in the core path)
- [ ] Any new indicator ships a parity test against `wickra-core`
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
