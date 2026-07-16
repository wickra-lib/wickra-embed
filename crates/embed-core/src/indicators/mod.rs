//! The verified no-alloc indicator subset.
//!
//! Each indicator here reproduces the exact floating-point recurrence of its
//! `wickra-core` counterpart — same operation order, same reseed cadence, same
//! division points — so its output is byte-for-byte identical, verified by the
//! `parity` tests. The window length is a const generic `N`; there is no runtime
//! buffer and no allocation.

mod atr;
mod ema;
mod roc;
mod rsi;
mod sma;

pub use atr::Atr;
pub use ema::Ema;
pub use roc::Roc;
pub use rsi::Rsi;
pub use sma::Sma;
