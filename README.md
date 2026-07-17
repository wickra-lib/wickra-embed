<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Embed — allocation-free, no_std streaming indicators for bare-metal and HFT, byte-for-byte identical to wickra-core" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-embed)
[![CI](https://github.com/wickra-lib/wickra-embed/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra-embed/actions/workflows/ci.yml)
[![CodeQL](https://github.com/wickra-lib/wickra-embed/actions/workflows/codeql.yml/badge.svg)](https://github.com/wickra-lib/wickra-embed/actions/workflows/codeql.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![OpenSSF Scorecard](https://img.shields.io/badge/OpenSSF-Scorecard-3b82f6)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-embed)
[![no_std](https://img.shields.io/badge/no__std-yes-success.svg)](docs/NO_STD.md)
[![byte-parity: wickra-core](https://img.shields.io/badge/byte--parity-wickra--core-success.svg)](docs/PARITY.md)
[![targets: thumbv7em / thumbv6m](https://img.shields.io/badge/targets-thumbv7em%20%7C%20thumbv6m-informational.svg)](docs/NO_STD.md)
[![Docs](https://img.shields.io/badge/docs-wickra.org-3b82f6)](https://wickra.org)

---

# wickra-embed

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

Early development (0.1.0, unreleased). The v0.1 line ships a verified no-alloc
subset of the indicator catalogue (`Sma`, `Ema`, `Rsi`, `Atr`, `Roc`); the
subset grows over time, each addition gated on a byte-parity test against
`wickra-core`. The core API and the C ABI handle contract are stable and pinned
by [golden tests](golden/).

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — the layers and the no-alloc design.
- [docs/NO_STD.md](docs/NO_STD.md) — no_std / no-alloc design, ports, panic handler, targets.
- [docs/INDICATORS.md](docs/INDICATORS.md) — the v0.1 subset and how to extend it.
- [docs/PARITY.md](docs/PARITY.md) — the byte-parity moat against `wickra-core`.
- [docs/C_ABI.md](docs/C_ABI.md) — the no-alloc C ABI handle contract.
- [docs/LATENCY.md](docs/LATENCY.md) — bounded per-update latency and how it is measured.

## Quickstart (Rust firmware)

Add the core with default features off — that is the `#![no_std]`, no-alloc build:

```toml
[dependencies]
embed-core = { git = "https://github.com/wickra-lib/wickra-embed", default-features = false }
```

Every indicator holds its whole state inline (a const-generic ring plus a few
scalars), so there is nothing to allocate — construct it on the stack or in a
`static` and feed it one data point at a time:

```rust
use embed_core::{Indicator, Sma};

// A fixed-window SMA(20): a 20-slot ring plus a running sum, entirely inline.
let mut sma = Sma::<20>::new();

for price in prices {
    match sma.update(price) {
        // Warm: `avg` is the byte-for-byte value wickra-core would produce.
        Some(avg) => act_on(avg),
        // Still warming up — SMA(N) emits its first value on the N-th input.
        None => {}
    }
}
```

`update` is `O(1)`, never allocates, and never panics on the hot path. `Ema`
takes a runtime period (`Ema::new(period)`); `Atr` consumes a
[`Candle`](crates/embed-core/src/ohlcv.rs) instead of a scalar price. See
[docs/INDICATORS.md](docs/INDICATORS.md) for the full subset.

## C ABI (no-alloc handles)

Firmware written in C or C++ links the C ABI, which — unlike every other Wickra
C ABI — **never calls `malloc`**. Handles are opaque and caller-allocated: query
the target-dependent size and alignment, place a buffer, and `init` into it.

```c
#include "wickra_embed.h"

alignas(16) unsigned char storage[512];      /* checked against the accessors */
WickraSma *sma = (WickraSma *) storage;
wickra_sma_init(sma);

double avg;
if (wickra_sma_update(sma, price, &avg) == WICKRA_EMBED_READY) {
    /* warm: use `avg` */
}
```

The full handle contract, return codes, and per-indicator surface are in
[docs/C_ABI.md](docs/C_ABI.md); a runnable sample is
[`examples/c/`](examples/c/).

## Parity and determinism

The moat is a single guarantee: the value computed on bare metal is
**byte-for-byte identical** to the value the std `wickra-core` computes on a
server — the same bits on a Cortex-M0, a Cortex-M4F, and an x86-64 host. IEEE-754
`f64` arithmetic is deterministic, so this reduces to performing the exact same
operations in the exact same order as `wickra-core`; a dev-dependency parity
suite asserts it with `f64::to_bits`, not a tolerance. See
[docs/PARITY.md](docs/PARITY.md).

## Latency

Every `update` is `O(1)` with fixed-size state and no allocation, so the
worst-case per-update cost has a hard ceiling — the property that matters for HFT
and hard-real-time firmware. The one source of variation, `Sma`'s bounded
rolling-sum reseed, is measured explicitly. Host nanoseconds and Cortex-M cycles
are tracked in [docs/LATENCY.md](docs/LATENCY.md) and [BENCHMARKS.md](BENCHMARKS.md).

## Targets and no_std

`embed-core` is `#![no_std]` and uses **neither `std` nor `alloc`** — no `Box`,
no `Vec`, no allocator anywhere. It is built on every change against, and is
byte-identical across:

| Target | Meaning |
|--------|---------|
| `thumbv7em-none-eabihf` | Cortex-M4F / M7, hardware FPU |
| `thumbv6m-none-eabi`    | Cortex-M0 / M0+, all soft-float |
| `x86_64-*` (host)       | tests, doctests, benches |

The bare-metal targets are pinned in `rust-toolchain.toml`. The design, the
panic-free hot path, and the `libm` math switch are covered in
[docs/NO_STD.md](docs/NO_STD.md).

## Project layout

```
crates/embed-core     #![no_std], no-alloc indicator core (Sma/Ema/Rsi/Atr/Roc)
crates/embed-bench     criterion per-update latency benches (host)
bindings/c             no-alloc C ABI (staticlib + cdylib) + generated header
examples/c             CMake/ctest C usage sample
examples/embedded      bare-metal Cortex-M example (thumbv7em, DWT cycle counts)
examples/host          host runner example
golden                 byte-parity fixtures replayed across targets
fuzz                   libFuzzer targets over the update path
docs                   NO_STD / INDICATORS / PARITY / C_ABI / LATENCY deep-dives
```

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
[BENCHMARKS.md](BENCHMARKS.md) and [docs/LATENCY.md](docs/LATENCY.md); the
Cortex-M cycle numbers land with the QEMU example run.

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
