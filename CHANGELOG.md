# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-09-23

A maintenance release: the embedded core and its C binding are unchanged. It
publishes the refreshed dependency tree and toolchain pins.

### Changed

- **Built on wickra-core 1.0.5.** The lock takes the indicator core's latest
  release; the `1.0` requirement already admitted it.
- **Third-party dependencies refreshed.** `Cargo.lock` takes 46 crates to their
  newest versions compatible with the Rust floor (the lock now resolves
  MSRV-aware, see below), run across the family in one pass so every repository
  resolves the same day's versions. No manifest changed.
- **The lockfile resolves for the Rust floor.** `.cargo/config.toml` sets
  `incompatible-rust-versions = "fallback"`, so `cargo update` takes the newest
  version the workspace's `rust-version` can build rather than the newest
  release -- the setting compile, copilot and shazam already carried, now
  family-wide. Without it, a routine refresh elsewhere in the family raised the
  icu crates to 2.3.0, which declares Rust 1.88, above a 1.86 floor. Re-resolved
  under it, the lock steps back to the newest versions the floor can build for
  `wasip2`, `wit-bindgen`.
- **The README's static badges are served by the organization** rather than
  hot-linked from shields.io, so they no longer break when shields is down.

## [0.1.1] - 2026-09-18

### Changed

- **Every README follows wickra's shape.** A cross-repo scan compared the
  heading skeleton of each README against wickra's and this repository's
  differed throughout. The root README opens as wickra's does (banner, badges,
  the one-liner, the ecosystem line, no separate H1), its Status names the
  current release, the License section carries wickra's wording and its
  `### Contribution` clause, and the shared sections run in wickra's order. The
  C ABI README is `Install`, `Quick start`, `Benchmark`, `Documentation`,
  `Security`, `Disclaimer`, `License` with the handle contract, return codes
  and byte-parity notes as subsections; `examples/README.md` lists the Rust and
  C examples the way wickra's does, `examples/c/README.md` carries the build
  matrix, and `fuzz/README.md` and the `## Editing the docs` section of
  `docs/README.md` exist as they do in wickra.

### Changed

- **The fuzz job runs the family's pinned nightly.** A cross-repo scan lined the
  24 wickra-lib repositories up; the only thing this one spelled differently was
  a rolling `nightly` for cargo-fuzz where the family pins `nightly-2026-07-01`
  by date, with the reason beside it. `CMAKE_C_STANDARD 11` stays: this is the
  family's C-only repository and the value is its own.

## [0.1.0] - 2026-09-14

### Changed

- **The repository has the family's shape.** SPDX-named licence copies under
  `LICENSES/`, a `docs/` index, the detailed issue and pull-request templates,
  Dependabot over the detached manifests, and a README with the quickstart
  first, a full build section, a Testing section and the ecosystem map.
  `rust-toolchain.toml` goes: CI pins its toolchains per job and installs the
  bare-metal targets it needs; a local checkout uses its own. The golden
  fixtures write warmup cells as `nan`, the family's convention.
- **CI and code scanning.** Pull requests build against `main` only; every
  action pin carries its patch-level version; the flake-resilience
  environment is set once; actionlint lints the workflows; CodSpeed measures
  the benches on every push; CodeQL has a config, a timeout and a C analysis
  of the sample; new jobs run the host example, the three repository
  consistency scripts (`scripts/check_*.py`) and osv-scanner against the
  committed `osv-scanner.toml`; every job has a timeout.
- **The release front.** `release.yml` refuses anything but a `v*` tag, checks
  the tag against the declared version, builds every artefact before a gate
  that requires the tagged commit's CI green, publishes `wickra-embed-core`
  idempotently, attaches the `.crate`, a CycloneDX SBOM and the C ABI archives
  (each with the header, the C++ wrapper and the licence texts) to a draft
  release, attests provenance for every asset and publishes last.
- **Crate metadata.** docs.rs builds with every feature; the licence texts sit
  inside the published crate and beside the C ABI; the dev-only parity oracle
  is `wickra-core` 1.0 (the released line; 0.9 was the last pre-release); the
  benches report to CodSpeed through `codspeed-criterion-compat`.

### Added

- **A C++ layer over the C ABI**, `bindings/c/include/wickra_embed.hpp`:
  one class per indicator with the handle's storage inline, a checked
  constructor instead of a hand-sized buffer, and an `Update { value, status,
  ready }` instead of an out-parameter. No heap, no exceptions, no RTTI. The C
  sample gets a C++ twin under the same `ctest`, and the wrapper ships in
  every C ABI release archive.
- `wickra_embed_core::indicators::CATALOGUE` names the verified subset, with a
  test pinning its count so an indicator cannot be added without its
  parity test, C ABI handle and documentation line.

### Fixed

- **The published crate carried a name the release could not upload.**
  `embed-core` is outside the org's crates.io token scope, which creates new
  crates under the `wickra-` prefix only; `cargo publish` on it returns 403
  at upload while `--dry-run` passes. It is now `wickra-embed-core`, the
  shape of every released sibling. The directory keeps its name; only the
  package and the `wickra_embed_core` path moved. The same audit ran across
  the family (xray paid for this with its first tag).
- **The bare-metal C ABI archives build.** The release built
  `libwickra_embed.a` for `thumbv6m` and `thumbv7em` with the default `std`
  feature, which those targets do not have, and CI never built the C crate
  for them. The release builds the two archives with `--no-default-features`
  and CI proves that build on both targets. A staticlib must carry a panic
  handler, so with `std` off the crate's own handler forwards to
  `wickra_embed_panic()`, a C function the firmware defines and that must not
  return; the generated header declares it and the C ABI docs give the
  firmware build command that actually works (`cargo rustc … --crate-type
  staticlib`; a bare-metal target cannot build the cdylib).

### Added

- `bindings/c`: a no-alloc C ABI (`wickra-embed-c`, built as `staticlib` +
  `cdylib`) exposing the indicator subset through caller-allocated, opaque
  handles — `wickra_<ind>_{size,align,init,update,reset,warmup,is_ready}` plus
  `wickra_embed_version`. The library never allocates; the caller places each
  handle on its own stack or in static storage. The cbindgen-generated header
  `include/wickra_embed.h` is committed and drift-checked.
- `wickra-embed-core`: the `#![no_std]`, allocation-free indicator core (`Sma`, `Ema`,
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

[Unreleased]: https://github.com/wickra-lib/wickra-embed/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/wickra-lib/wickra-embed/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wickra-lib/wickra-embed/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wickra-lib/wickra-embed/releases/tag/v0.1.0
