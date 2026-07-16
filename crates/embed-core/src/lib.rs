//! Allocation-free, `#![no_std]` streaming technical indicators.
//!
//! `embed-core` runs the Wickra indicator math where there is no operating
//! system and no heap — microcontrollers, FPGA soft-cores, HFT co-processors.
//! Every [`Indicator`] update is O(1) with a bounded worst case, keeps its state
//! inline in fixed-capacity buffers (no allocation), and produces the
//! **byte-for-byte identical** value the std [`wickra-core`] library produces on
//! a server.
//!
//! [`wickra-core`]: https://crates.io/crates/wickra-core
//!
//! # Design
//!
//! - **No heap.** Windowed indicators store their history in a const-generic
//!   [`Ring`](ring::Ring) or a fixed array; nothing touches an allocator. The
//!   window length is the const parameter `N` (e.g. [`Sma<20>`](Sma)).
//! - **No panics in the update path.** A panic on a microcontroller is a reset,
//!   so the core never uses `unwrap`/`expect`/`unreachable!` on the hot path;
//!   warmup and undefined states are expressed as `None`.
//! - **Deterministic to the bit.** The floating-point work is done in the exact
//!   same order as `wickra-core` (rolling-sum add/subtract order, the periodic
//!   reseed, the final `sum / period` division), so the result is identical on
//!   `thumbv7em-none-eabihf`, `thumbv6m-none-eabi` and an x86-64 host alike. The
//!   few transcendental ops (`abs`, `max`, fused multiply-add) route through a
//!   single [`math`] switch point — std on a host, `libm` on bare metal — whose
//!   two implementations are IEEE-754 identical.
//!
//! # Example
//!
//! ```
//! use embed_core::{Indicator, Sma};
//!
//! let mut sma = Sma::<3>::new();
//! let mut last = None;
//! for i in 0..10 {
//!     last = sma.update(100.0 + f64::from(i));
//! }
//! assert!(last.is_some());
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

pub mod indicators;
mod math;
pub mod ohlcv;
pub mod ring;
pub mod traits;

pub use indicators::{Atr, Ema, Roc, Rsi, Sma};
pub use ohlcv::Candle;
pub use traits::Indicator;

/// Crate version string (`CARGO_PKG_VERSION`), for the C ABI and diagnostics.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
