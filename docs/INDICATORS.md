# The v0.1 indicator subset

`embed-core` ships a small, verified, allocation-free subset of the Wickra
indicator catalogue. Each is a const-generic (or runtime-period) struct
implementing the [`Indicator`] trait, byte-for-byte identical to its
`wickra-core` counterpart (see [PARITY.md](PARITY.md)).

## The subset

| Indicator | Type | Input | Window | First output after | `math` ops used |
|-----------|------|-------|--------|--------------------|-----------------|
| `Sma<N>`  | Simple Moving Average | `f64` | `N` (ring) | `N` inputs | none (`+ - * /`) |
| `Ema`     | Exponential Moving Average | `f64` | runtime period, no buffer | `period` inputs | `mul_add` |
| `Rsi<N>`  | Relative Strength Index (Wilder) | `f64` | none (incremental) | `N + 1` inputs | `mul_add` |
| `Atr<N>`  | Average True Range (Wilder) | `Candle` | none (incremental) | `N` candles | `mul_add`, `abs`, `max` |
| `Roc<N>`  | Rate of Change (%) | `f64` | `N` (ring) | `N + 1` inputs | none (`+ - * /`) |

`Ema` uses a runtime `period` because it needs no window buffer (only a running
warmup sum), so it stays allocation-free without a const generic. The others take
the window length as the const parameter `N`, so their storage is fixed at
compile time.

## The `Indicator` contract

Every indicator implements the same trait:

```rust
pub trait Indicator {
    type Input: Copy;                                  // f64 or Candle
    fn update(&mut self, input: Self::Input) -> Option<f64>;  // None during warmup
    fn reset(&mut self);
    fn warmup_period(&self) -> usize;
    fn is_ready(&self) -> bool;
    fn name(&self) -> &'static str;
}
```

`update` is O(1) and returns `None` until warmup is complete, then `Some(value)`.
`Input` is `Copy` (a price `f64` or a [`Candle`]) so the update path never clones
or allocates.

```rust
use embed_core::{Indicator, Sma};

let mut sma = Sma::<20>::new();
for price in prices {
    if let Some(v) = sma.update(price) {
        // v is a defined average
    }
}
```

## Non-finite inputs

`Sma`, `Ema` and `Roc` ignore non-finite inputs (NaN / ±∞): they return the last
value and leave state untouched, matching `wickra-core`. `Atr` reads a `Candle`,
whose fields the caller is trusted to keep finite (the bindings validate up
front).

## Extending the subset

To add an indicator:

1. Add a `#![no_std]`, allocation-free struct under
   `crates/embed-core/src/indicators/`, implementing `Indicator`. Use a
   const-generic array or a running scalar for state — never `Vec`/`Box`.
2. Match the exact f64 operation order of `wickra-core/src/indicators/<name>.rs`
   (see [PARITY.md](PARITY.md)) so byte-parity holds.
3. If it needs a transcendental op, route it through the `math` module and add a
   `std == libm` check — or, if `libm` diverges for that op, leave the indicator
   out of the no_std subset and document why.
4. Ship a byte-parity test against `wickra-core`.

An indicator that cannot be made allocation-free, or whose `libm` path diverges
from `std`, does not belong in the subset; that is a deliberate boundary, not a
gap to paper over.

[`Indicator`]: ../crates/embed-core/src/traits.rs
[`Candle`]: ../crates/embed-core/src/ohlcv.rs
