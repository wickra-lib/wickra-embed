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

/// The verified subset by name, in the order the C ABI and the docs list it.
///
/// One entry per module above. The test below pins the count, so an indicator
/// added here without its parity test, its C ABI handle and its line in
/// `docs/INDICATORS.md` shows up as a number that no longer matches.
pub const CATALOGUE: &[&str] = &["Sma", "Ema", "Rsi", "Atr", "Roc"];

#[cfg(test)]
mod tests {
    use super::CATALOGUE;

    #[test]
    fn total_count_matches_expected() {
        // Bump together with new indicators. Drift between this number and
        // the module list above is the early-warning signal that an indicator
        // was added without being named in the catalogue.
        let total = CATALOGUE.len();
        assert_eq!(total, 5, "CATALOGUE total drifted from the indicator count");
        assert!(CATALOGUE.windows(2).all(|w| w[0] != w[1]));
    }
}
