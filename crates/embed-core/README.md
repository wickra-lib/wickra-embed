# embed-core

Allocation-free, `#![no_std]` streaming technical indicators — byte-for-byte
identical to [wickra-core](https://crates.io/crates/wickra-core).

`embed-core` is the computational heart of
[wickra-embed](https://github.com/wickra-lib/wickra-embed): a small, verified,
no-alloc subset of the Wickra indicator catalogue (`Sma`, `Ema`, `Rsi`, `Atr`,
`Roc`) for bare-metal and HFT hardware. Every update is O(1) with a bounded
worst case, keeps its state inline in fixed-capacity buffers (no heap), and
produces the exact same bits the std `wickra-core` produces on a server —
verified by a byte-parity test suite.

```rust
use embed_core::{Indicator, Sma};

let mut sma = Sma::<20>::new();      // window length is the const generic
let mut latest = None;
for price in prices {
    latest = sma.update(price);       // None during warmup, then Some(value)
}
```

- **No heap.** Const-generic ring buffers; nothing touches an allocator.
- **No panics on the hot path.** Warmup and undefined states are `None`.
- **Deterministic.** Same bits on `thumbv7em`, `thumbv6m`, and an x86-64 host.

The bare-metal build uses `--no-default-features`; the default `std` feature is
for host tests and benches only. `wickra-core` is a **dev-dependency parity
oracle**, never linked into the no_std build.

Dual-licensed under `MIT OR Apache-2.0`.
