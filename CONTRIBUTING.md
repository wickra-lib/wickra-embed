# Contributing to wickra-embed

Thanks for your interest. Issues, bug reports, ideas and pull requests are all
welcome at <https://github.com/wickra-lib/wickra-embed>. For larger changes,
open an issue first so we can agree on the approach.

## Orientation

- The core — the `Indicator` contract, the const-generic no-alloc ring buffer,
  and the verified subset of indicators (`Sma`, `Ema`, `Rsi`, `Atr`, `Roc`, …) —
  lives in `crates/embed-core`. It is `#![no_std]` and allocation-free: no
  `Box`, no `Vec`, no heap. `wickra-core` appears only as a **dev-dependency**,
  the parity oracle the no-alloc values are checked byte-for-byte against.
- The C ABI lives in `bindings/c`: a no-alloc, stack-handle surface — the one
  place `unsafe` is re-allowed — with a cbindgen-generated header.
- Examples live under `examples/` (a host CSV runner, a C usage sample, and a
  `#![no_std] #![no_main]` QEMU/Cortex-M demo).

## The dev loop

Every change runs green locally before a commit:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
# The no_std core must build without std on the bare-metal targets:
cargo build -p embed-core --no-default-features --target thumbv7em-none-eabihf
cargo build -p embed-core --no-default-features --target thumbv6m-none-eabi
# Host tests, including byte-parity against wickra-core:
cargo test -p embed-core --all-features
cargo deny check
```

`cargo fmt --all` and the `clippy -D warnings` gate are enforced in CI, and the
no_std core is built on both Cortex-M targets on every PR.

## Conventions

- **Commits are signed** and follow Conventional Commits (`feat:`, `fix:`,
  `chore:`, `docs:`…). One logical change per commit. Open a PR against `main`;
  do not push to `main` directly.
- **All public artifacts are in English** — code, comments, commit messages, PR
  titles and bodies, issues and docs.
- **No secrets, ever.**
- **Production code only** — no mocks outside `#[cfg(test)]`, no TODO stubs, and
  no defensive branches that can never run (they fail coverage). No `panic!`,
  `unwrap` or `expect` in the no_std core path — a panic on an MCU is a reset;
  return `Result`/`Option` instead.

## Adding an indicator

Every new indicator is a `#![no_std]`, allocation-free struct implementing
`Indicator`, added under `crates/embed-core/src/indicators/`, and **must ship a
parity test** in `crates/embed-core/tests/parity.rs` proving its output is
byte-for-byte identical to the corresponding `wickra-core` indicator. Preserve
the exact f64 operation order of the reference (rolling-sum add/subtract order,
the periodic reseed, the final division) so the byte-parity holds on every
target.

## Developer Certificate of Origin

Contributions are accepted under the [DCO](DCO); sign off your commits with
`git commit -s`. By contributing you agree your work is dual-licensed under
`MIT OR Apache-2.0`.
