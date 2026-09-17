<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Embed — allocation-free, no_std streaming indicators for bare-metal and HFT, byte-for-byte identical to wickra-core" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-embed)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/ci.svg)](https://github.com/wickra-lib/wickra-embed/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/codeql.svg)](https://github.com/wickra-lib/wickra-embed/actions/workflows/codeql.yml)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/release.svg)](https://github.com/wickra-lib/wickra-embed/releases/latest)
[![crates.io](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/crates.svg)](https://crates.io/crates/wickra-embed-core)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-embed)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/best-practices.svg)](https://www.bestpractices.dev)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/provenance.svg)](https://github.com/wickra-lib/wickra-embed/attestations)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/docs.svg)](https://embed.wickra.org)
[![no_std](https://img.shields.io/badge/no__std-yes-success.svg)](docs/NO_STD.md)
[![byte-parity: wickra-core](https://img.shields.io/badge/byte--parity-wickra--core-success.svg)](docs/PARITY.md)
[![targets: thumbv7em / thumbv6m](https://img.shields.io/badge/targets-thumbv7em%20%7C%20thumbv6m-informational.svg)](docs/NO_STD.md)

---

**Allocation-free, `#![no_std]` streaming indicators for bare-metal and HFT — byte-for-byte identical to [wickra-core](https://github.com/wickra-lib/wickra).**

> **▶ Live demos:** the backtester compiled to WebAssembly, an equity curve building bar by bar — **[backtest-live.wickra.org](https://backtest-live.wickra.org)**;
> one StrategySpec side by side in Python, Rust, JS and Go — **[playground.wickra.org](https://playground.wickra.org)**;
> all 514 indicators of the core over a real Binance feed — **[live.wickra.org](https://live.wickra.org)**. Zero backend, all of them.

**Part of the [Wickra ecosystem](#ecosystem):** the same indicator core and ten-language binding surface also power [wickra](https://github.com/wickra-lib/wickra), [wickra-backtest](https://github.com/wickra-lib/wickra-backtest), [wickra-pico](https://github.com/wickra-lib/wickra-pico) and 20 more — see [the full list](https://github.com/wickra-lib).

`wickra-embed` runs the Wickra indicator math where there is no operating system
and no heap: microcontrollers, FPGA soft-cores, HFT co-processors. Every update
is O(1) with a bounded worst-case latency, uses fixed-capacity buffers (no
allocation), and produces the **byte-for-byte identical** value the std
`wickra-core` produces on a server — verified by a parity test suite.

```toml
[dependencies]
wickra-embed-core = { version = "0.1", default-features = false }
```

```rust
use wickra_embed_core::{Indicator, Sma};

let mut sma = Sma::<20>::new();
for price in [101.0, 102.5, 101.8] {
    if let Some(v) = sma.update(price) {
        // one value per input once the warmup is over, no heap touched
        let _ = v;
    }
}
```

## Status

**0.1.0 — the current release.** The v0.1 line ships a verified no-alloc subset
of the indicator catalogue (`Sma`, `Ema`, `Rsi`, `Atr`, `Roc`); the subset grows
over time, each addition gated on a byte-parity test against `wickra-core`. The
core API and the C ABI handle contract are stable and pinned by [golden
tests](golden/).

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
wickra-embed-core = { version = "0.1", default-features = false }
```

Every indicator holds its whole state inline (a const-generic ring plus a few
scalars), so there is nothing to allocate — construct it on the stack or in a
`static` and feed it one data point at a time:

```rust
use wickra_embed_core::{Indicator, Sma};

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

`wickra-embed-core` is `#![no_std]` and uses **neither `std` nor `alloc`** — no `Box`,
no `Vec`, no allocator anywhere. It is built on every change against, and is
byte-identical across:

| Target | Meaning |
|--------|---------|
| `thumbv7em-none-eabihf` | Cortex-M4F / M7, hardware FPU |
| `thumbv6m-none-eabi`    | Cortex-M0 / M0+, all soft-float |
| `x86_64-*` (host)       | tests, doctests, benches |

The bare-metal targets are added with `rustup target add`; CI installs them
per job. The design, the
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

## Building everything from source

```bash
cargo build --workspace
cargo test  --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings

# The no_std core on the bare-metal targets (rustup target add them once):
cargo build -p wickra-embed-core --no-default-features --target thumbv7em-none-eabihf
cargo build -p wickra-embed-core --no-default-features --target thumbv6m-none-eabi

# The C ABI staticlib for a target, and the C sample against it:
cargo rustc -p wickra-embed-c --release --crate-type staticlib
cmake -S examples/c -B examples/c/build && cmake --build examples/c/build && ctest --test-dir examples/c/build
```

## Testing

Run the suites with the commands in
[Building everything from source](#building-everything-from-source).

- **`wickra-embed-core`** — unit tests per indicator, the handle contract
  (`contract.rs`), property tests over the update path, and the byte-parity
  suite (`parity.rs`) that folds the same inputs through `wickra-core` and
  asserts identical bits. The golden fixtures in `golden/` are the anchor: the
  committed columns are what every target must reproduce.
- **C ABI** — `examples/c` builds and runs the sample against the staticlib
  through `ctest`; the header is regenerated with cbindgen and diffed in CI so
  it cannot drift from the exported symbols.
- **Bare metal** — the no_std core builds for `thumbv7em-none-eabihf` and
  `thumbv6m-none-eabi`, the no-alloc guard proves no allocator symbol is linked,
  and `examples/embedded` runs under QEMU.
- **Fuzz** — `fuzz/` holds libFuzzer targets over the update path; CI runs each
  for a short smoke.

## Requirements

- Rust 1.86+ (MSRV). The bare-metal targets `thumbv7em-none-eabihf` and
  `thumbv6m-none-eabi` come from `rustup target add`.
- Optional: `qemu-system-arm` to run the Cortex-M example, `cmake` + a C toolchain
  to build the C usage sample.

## Benchmarks

Per-update latency (host ns and MCU cycles) is tracked in
[BENCHMARKS.md](BENCHMARKS.md) and [docs/LATENCY.md](docs/LATENCY.md); the
Cortex-M cycle numbers land with the QEMU example run.

## Ecosystem

Part of the [Wickra](https://github.com/wickra-lib/wickra) family — each one a
data-driven core with a CLI and the same ten-language binding surface:

- [**wickra**](https://github.com/wickra-lib/wickra) — main library (Rust core + Python / Node.js / WASM bindings + a C ABI for C / C++ / C# / Go / Java / R)
- [**wickra-playground**](https://github.com/wickra-lib/wickra-playground) — a polyglot strategy playground: one StrategySpec live side by side in Python, Rust, JS and Go, entirely in the browser
- [**wickra-exchange**](https://github.com/wickra-lib/wickra-exchange) — unified market-data + execution across ten crypto exchanges
- [**wickra-backtest**](https://github.com/wickra-lib/wickra-backtest) — event-driven backtester over the Wickra core
- [**wickra-terminal**](https://github.com/wickra-lib/wickra-terminal) — the trading terminal: a TUI and a browser renderer over the stack
- [**wickra-screener**](https://github.com/wickra-lib/wickra-screener) — parallel multi-symbol screening over 514 streaming indicators
- [**wickra-radar**](https://github.com/wickra-lib/wickra-radar) — perp-universe alert radar: OI delta, funding flip, book imbalance, liquidation clusters, OI/price divergence
- [**wickra-copilot**](https://github.com/wickra-lib/wickra-copilot) — local market copilot grounded in real order-book, liquidation and funding microstructure
- [**wickra-shazam**](https://github.com/wickra-lib/wickra-shazam) — match an asset's current microstructure fingerprint against its entire history
- [**wickra-benchmark**](https://github.com/wickra-lib/wickra-benchmark) — reproducible, golden-verified benchmark suite — recompute any (strategy, dataset, report) in ten languages and confirm it byte-for-byte
- [**wickra-strategy-ci**](https://github.com/wickra-lib/wickra-strategy-ci) — Jest for trading strategies: golden-pin the report, catch regressions in CI, property-test against fuzzed data
- [**wickra-verify**](https://github.com/wickra-lib/wickra-verify) — confirm or refute a claimed backtest report against its strategy and data, in ten languages
- [**wickra-proof**](https://github.com/wickra-lib/wickra-proof) — Proof-of-Backtest: deterministic (spec, data) → report + blake3 hash, recomputable byte-for-byte in ten languages
- [**wickra-zk**](https://github.com/wickra-lib/wickra-zk) — prove a backtest zero-knowledge — on-chain-verifiable performance without revealing the data or the strategy
- [**wickra-impact**](https://github.com/wickra-lib/wickra-impact) — the backtester that knows you would have moved the market: agent-based fills on the real historical L2 order book
- [**wickra-darwin**](https://github.com/wickra-lib/wickra-darwin) — evolutionary strategy search at millions of backtests per second, mutating and crossing JSON specs across the 514-indicator space
- [**wickra-gym**](https://github.com/wickra-lib/wickra-gym) — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for deterministic RL rollouts
- [**wickra-feature-store**](https://github.com/wickra-lib/wickra-feature-store) — OHLCV and microstructure streams into ML-ready feature matrices over 514 O(1) streaming indicators
- [**wickra-genome**](https://github.com/wickra-lib/wickra-genome) — a vector database of the whole market: every asset a 514-dim live vector, for similarity search, clustering and anomaly detection
- [**wickra-timemachine**](https://github.com/wickra-lib/wickra-timemachine) — scrub the whole market like a video — every symbol, full order book, rewound to any moment via deterministic re-fold
- [**wickra-synth**](https://github.com/wickra-lib/wickra-synth) — deterministic synthetic market microstructure: OHLCV, order book, trades and funding from a single seed
- [**wickra-compile**](https://github.com/wickra-lib/wickra-compile) — compile a strategy spec into a standalone deployable: a WASM module, a self-contained binary, or a `no_std` artifact
- [**wickra-pico**](https://github.com/wickra-lib/wickra-pico) — the O(1) indicator core running bare-metal on a $5 Raspberry Pi Pico — the LED blinks on the EMA cross

Docs at [docs.wickra.org](https://docs.wickra.org); the marketing site and
in-browser demo at [wickra.org](https://wickra.org).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Every new indicator ships a byte-parity
test against `wickra-core`.

## Security

See [SECURITY.md](SECURITY.md) and [THREAT_MODEL.md](THREAT_MODEL.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option. Use it, fork it, modify it, redistribute it — commercially or
not — file issues, send pull requests; all welcome.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Disclaimer

`wickra-embed` computes technical indicators. It is analysis software, not
financial advice, and comes with no warranty. Trading carries risk; you are
responsible for your own decisions.

---

<p align="center">
  <a href="https://github.com/wickra-lib/wickra-embed">
    <img alt="GitHub stars" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/stars.svg">
  </a>
  <a href="https://github.com/wickra-lib/wickra-embed/network/members">
    <img alt="GitHub forks" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/forks.svg">
  </a>
  <a href="https://github.com/wickra-lib/wickra-embed/issues">
    <img alt="GitHub issues" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/issues.svg">
  </a>
</p>

<p align="center">
  Built on <a href="https://github.com/wickra-lib/wickra">Wickra</a>. If it saved you time, the cheapest way to say thanks is to ⭐ the repo.
</p>

<p align="center">
  <img alt="wickra-embed star history" width="640"
       src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-embed/star-history.svg">
</p>
