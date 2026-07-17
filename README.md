<p align="center">
  <img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp" alt="Wickra" width="100%">
</p>

# wickra-embed

[![CI](https://github.com/wickra-lib/wickra-embed/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra-embed/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![no_std](https://img.shields.io/badge/no__std-yes-success.svg)](docs/NO_STD.md)
[![targets: thumbv7em / thumbv6m](https://img.shields.io/badge/targets-thumbv7em%20%7C%20thumbv6m-informational.svg)](docs/NO_STD.md)
[![byte-parity: wickra-core](https://img.shields.io/badge/byte--parity-wickra--core-success.svg)](docs/PARITY.md)

**Allocation-free, `#![no_std]` streaming indicators for bare-metal and HFT — byte-for-byte identical to [wickra-core](https://github.com/wickra-lib/wickra).**

`wickra-embed` runs the Wickra indicator math where there is no operating system
and no heap: microcontrollers, FPGA soft-cores, HFT co-processors. Every update
is O(1) with a bounded worst-case latency, uses fixed-capacity buffers (no
allocation), and produces the **byte-for-byte identical** value the std
`wickra-core` produces on a server — verified by a parity test suite.

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same
> indicator math powers the full-fat [wickra](https://github.com/wickra-lib/wickra)
> library and its downstream tools; `wickra-embed` is the no-alloc, bare-metal
> distillation of that math.

## Status

Pre-release, built out in phases (see [ROADMAP.md](ROADMAP.md)). The v0.1 line
ships a verified no-alloc subset of the indicator catalogue (`Sma`, `Ema`,
`Rsi`, `Atr`, `Roc`); the subset grows over time, each addition gated on a
byte-parity test against `wickra-core`.

## Why

- **No heap.** Fixed-capacity, const-generic ring buffers. Nothing touches an
  allocator, so it runs on a Cortex-M0 with a few KB of RAM.
- **Bounded latency.** Every `update` is O(1); the only variation is a periodic
  rolling-sum reseed, itself bounded.
- **Deterministic.** The same input yields the same bytes on `thumbv7em`,
  `thumbv6m` and an x86-64 host — and the same bytes as `wickra-core`. See
  [docs/PARITY.md](docs/PARITY.md); determinism is covered in [docs/NO_STD.md](docs/NO_STD.md).
- **Usable from C.** A no-alloc C ABI with stack-allocated handles, for firmware
  written in C/C++.

## Building from source

```bash
# Host build + tests (parity against wickra-core):
cargo test -p embed-core --all-features

# The no_std core on bare-metal targets (pulled automatically via rust-toolchain.toml):
cargo build -p embed-core --no-default-features --target thumbv7em-none-eabihf
cargo build -p embed-core --no-default-features --target thumbv6m-none-eabi
```

## Requirements

- Rust 1.86+ (MSRV). The bare-metal targets `thumbv7em-none-eabihf` and
  `thumbv6m-none-eabi` are pinned in `rust-toolchain.toml`.
- Optional: `qemu-system-arm` to run the Cortex-M example, `cmake` + a C toolchain
  to build the C usage sample.

## Benchmarks

Per-update latency (host ns and MCU cycles) is tracked in
[BENCHMARKS.md](BENCHMARKS.md); the numbers land as the bench crate and the
Cortex-M example are built out.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — the layers and the no-alloc design.
- [docs/NO_STD.md](docs/NO_STD.md) — no_std / no-alloc design, ports, panic handler, targets.
- [docs/INDICATORS.md](docs/INDICATORS.md) — the v0.1 subset and how to extend it.
- [docs/PARITY.md](docs/PARITY.md) — the byte-parity moat against `wickra-core`.
- [docs/C_ABI.md](docs/C_ABI.md) — the no-alloc C ABI handle contract.
- [docs/LATENCY.md](docs/LATENCY.md) — bounded per-update latency and how it is measured.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Every new indicator ships a byte-parity
test against `wickra-core`.

## Security

See [SECURITY.md](SECURITY.md) and [THREAT_MODEL.md](THREAT_MODEL.md).

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. Unless you explicitly state otherwise, any contribution
intentionally submitted for inclusion in this work, as defined in the Apache-2.0
license, shall be dual-licensed as above, without any additional terms or
conditions.

## Disclaimer

`wickra-embed` computes technical indicators. It is analysis software, not
financial advice, and comes with no warranty. Trading carries risk; you are
responsible for your own decisions.
