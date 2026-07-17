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
- Repository scaffolding: Cargo workspace, supply-chain configuration
  (`deny.toml`, `osv-scanner.toml`, `lychee.toml`), lint configuration
  (`clippy.toml`), `rust-toolchain.toml` pinning the bare-metal cross-compile
  targets, `repo-metadata.toml`, governance docs, the `.github` tree
  (issue/PR templates, `setup-rust`, `sync-metadata.py`, dependabot), and dual
  `MIT OR Apache-2.0` licensing.

[Unreleased]: https://github.com/wickra-lib/wickra-embed/commits/main
