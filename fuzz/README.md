# Fuzzing Wickra Embed

[`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) harnesses for the parsing and stateful entry points of Wickra Embed. Fuzzing requires a nightly Rust toolchain; CI runs every target for 30 seconds on the family's pinned `nightly-2026-07-01`.

## Setup

```bash
cargo install cargo-fuzz
rustup toolchain install nightly-2026-07-01
```

The date is the family's fuzz nightly, pinned in `ci.yml`: a rolling `nightly`
regressed with a codegen ICE unrelated to this code, so every repository moves
the date together, on purpose.

## Targets

| Target | What it exercises |
| --- | --- |
| `sma_update` | The scalar-input indicators (`Sma`, `Rsi`, `Roc`) that share the raw-f64 decode path. |
| `ema_update` | `Ema` on its own: it carries a running-average state that (unlike the windowed indicators) never resets, so it is the one most sensitive to a pathological input drifting the accumulator. |
| `ohlc_update` | The candle-input `Atr`. |

## Run

```bash
# From the repository root:
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu sma_update
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu ema_update
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu ohlc_update
```

Each run continues until a crash is found or it is interrupted. A short
time-boxed smoke run is what CI does:

```bash
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu sma_update -- -max_total_time=30
```

The expectation for every target is that it never panics: malformed or
adversarial input must surface as an `Err` or an in-band error, never a crash.
