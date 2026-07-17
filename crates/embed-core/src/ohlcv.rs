//! The [`Candle`] OHLC bar — the `Copy` input type for candle indicators.

use crate::math;

/// One OHLCV bar.
///
/// Field order and semantics match `wickra_core::ohlcv::Candle`, so a candle
/// built here feeds the parity oracle unchanged. `Copy` keeps the indicator
/// update path allocation- and clone-free.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    /// Bar open price.
    pub open: f64,
    /// Bar high price.
    pub high: f64,
    /// Bar low price.
    pub low: f64,
    /// Bar close price.
    pub close: f64,
    /// Bar volume.
    pub volume: f64,
    /// Bar timestamp (caller-defined epoch / resolution).
    pub timestamp: i64,
}

impl Candle {
    /// Construct a candle from its fields. Unlike the std `Candle::new`, this is
    /// a plain `const` constructor with no validation: on bare metal the caller
    /// (the C ABI, the firmware) is trusted to pass finite, ordered OHLC values,
    /// and a validating constructor would add a panic path the core forbids.
    #[must_use]
    pub const fn new(
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        timestamp: i64,
    ) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        }
    }

    /// True range for this bar given the previous close, identical to
    /// `wickra-core`: `max(high - low, |high - prev|, |low - prev|)`, or just
    /// `high - low` for the first bar.
    #[must_use]
    pub fn true_range(&self, prev_close: Option<f64>) -> f64 {
        let hl = self.high - self.low;
        match prev_close {
            Some(prev) => {
                let hp = math::abs(self.high - prev);
                let lp = math::abs(self.low - prev);
                math::max(math::max(hl, hp), lp)
            }
            None => hl,
        }
    }
}
