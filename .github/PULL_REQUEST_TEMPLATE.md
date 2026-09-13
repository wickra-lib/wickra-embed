<!-- Keep it short. One logical change per PR.

     Changing the release process, the C ABI's handle contract, or the public
     API? There is a longer template that asks what such a change has to answer:
     reopen this pull request with ?template=detailed.md appended to the URL.
     GitHub offers no picker for a second template, so it is only reachable that
     way. -->

## What

<!-- What does this change and why? -->

## Checklist

- [ ] `cargo fmt --all` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` are clean
- [ ] The no_std core builds on the bare-metal targets (`cargo build -p wickra-embed-core --no-default-features --target thumbv7em-none-eabihf` and `thumbv6m-none-eabi`)
- [ ] `cargo test -p wickra-embed-core --all-features` passes, including the byte-parity tests against `wickra-core`
- [ ] `cargo deny check` is clean
- [ ] Determinism preserved (fixed f64 operation order; no alloc, HashMap, time or RNG in the core path)
- [ ] Any new indicator ships a parity test against `wickra-core`
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
