# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `bindings/c`: a no-alloc C ABI (`wickra-embed-c`, built as `staticlib` +
  `cdylib`) exposing the indicator subset through caller-allocated, opaque
  handles — `wickra_<ind>_{size,align,init,update,reset,warmup,is_ready}` plus
  `wickra_embed_version`. The library never allocates; the caller places each
  handle on its own stack or in static storage. The cbindgen-generated header
  `include/wickra_embed.h` is committed and drift-checked.
- `embed-core`: the `#![no_std]`, allocation-free indicator core (`Sma`, `Ema`,
  `Rsi`, `Atr`, `Roc`), byte-for-byte identical to `wickra-core`, cross-built for
  `thumbv7em-none-eabihf` and `thumbv6m-none-eabi`.
- `golden/`: byte-parity fixtures generated once from `wickra-core` and replayed
  across the host and both bare-metal targets, so any divergence fails CI.
- `fuzz/`: libFuzzer targets over the scalar and OHLC update paths (never panic,
  every warm output finite, `is_ready` monotonic); `crates/embed-bench`:
  criterion per-update latency benches including the rolling-sum reseed cycle.
- `examples/`: a bare-metal Cortex-M example reporting DWT cycle counts on
  `thumbv7em`, a CMake/ctest C usage sample over the no-alloc handle ABI, and a
  host runner.
- CI/CD: `ci.yml` (fmt, clippy on both feature sets, no_std cross-builds for both
  targets, an allocation-symbol leak guard, a three-OS host test matrix, MSRV,
  coverage, `cargo-deny`, fuzz-smoke, the C ABI on three OS with header-drift,
  the bare-metal example build, and link checking), plus CodeQL, OpenSSF
  Scorecard, zizmor, lychee, a nightly criterion bench, a metadata audit, and a
  USER-gated `release.yml` (crates.io publish, per-target static-lib archives,
  CycloneDX SBOM, and build provenance).
- `docs/`: deep-dive documentation — `NO_STD.md`, `INDICATORS.md`, `PARITY.md`,
  `C_ABI.md`, `LATENCY.md` — alongside `ARCHITECTURE.md` and a finalized README.
- Repository scaffolding: Cargo workspace, supply-chain configuration
  (`deny.toml`, `osv-scanner.toml`, `lychee.toml`), lint configuration
  (`clippy.toml`), `rust-toolchain.toml` pinning the bare-metal cross-compile
  targets, `repo-metadata.toml`, governance docs, the `.github` tree
  (issue/PR templates, `setup-rust`, `sync-metadata.py`, dependabot), and dual
  `MIT OR Apache-2.0` licensing.

[Unreleased]: https://github.com/wickra-lib/wickra-embed/commits/main
