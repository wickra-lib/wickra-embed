//! Average True Range (Wilder) — const-generic, no-alloc, byte-parity with
//! `wickra_core::Atr`.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Average True Range over a Wilder period `N`.
///
/// Seeds with the mean of the first `N` true ranges, then applies Wilder
/// smoothing `avg.mul_add(N - 1, tr) / N`, identical to `wickra_core::Atr`. The
/// first value appears after `N` candles. Input is a [`Candle`]; only high, low
/// and the previous close are read.
///
/// # Example
///
/// ```
/// use embed_core::{Atr, Candle, Indicator};
///
/// let mut atr = Atr::<5>::new();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let c = Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i));
///     last = atr.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Atr<const N: usize> {
    n_minus_1: f64,
    inv_period: f64,
    prev_close: Option<f64>,
    /// Running sum and count of the seed true ranges, replacing wickra-core's
    /// `Vec<f64>` seed buffer. Incremental summation in arrival order is the
    /// identical fold to `seed_buf.iter().sum()`, so the seed average matches.
    seed_count: usize,
    seed_sum: f64,
    avg: f64,
    seeded: bool,
}

impl<const N: usize> Atr<N> {
    /// Construct a new ATR over a Wilder period of `N`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            n_minus_1: (N - 1) as f64,
            inv_period: 1.0 / N as f64,
            prev_close: None,
            seed_count: 0,
            seed_sum: 0.0,
            avg: 0.0,
            seeded: false,
        }
    }

    /// Current value if seeded.
    #[must_use]
    pub const fn value(&self) -> Option<f64> {
        if self.seeded {
            Some(self.avg)
        } else {
            None
        }
    }
}

impl<const N: usize> Default for Atr<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Indicator for Atr<N> {
    type Input = Candle;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let tr = candle.true_range(self.prev_close);
        self.prev_close = Some(candle.close);

        if self.seeded {
            let new_avg = crate::math::mul_add(self.avg, self.n_minus_1, tr) * self.inv_period;
            self.avg = new_avg;
            return Some(new_avg);
        }

        self.seed_sum += tr;
        self.seed_count += 1;
        if self.seed_count == N {
            let seed = self.seed_sum / N as f64;
            self.avg = seed;
            self.seeded = true;
            return Some(seed);
        }
        None
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn warmup_period(&self) -> usize {
        N
    }

    fn is_ready(&self) -> bool {
        self.seeded
    }

    fn name(&self) -> &'static str {
        "atr"
    }
}

#[cfg(test)]
mod tests {
    use super::Atr;
    use crate::ohlcv::Candle;
    use crate::traits::Indicator;

    fn candle(i: i64) -> Candle {
        let base = 100.0 + i as f64;
        Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i)
    }

    #[test]
    fn warmup_then_value() {
        let mut atr = Atr::<5>::new();
        for i in 0..4 {
            assert_eq!(atr.update(candle(i)), None);
        }
        let v = atr.update(candle(4));
        assert!(v.is_some());
        assert!(atr.is_ready());
        assert_eq!(atr.warmup_period(), 5);
    }

    #[test]
    fn reset_clears_state() {
        let mut atr = Atr::<5>::new();
        for i in 0..20 {
            atr.update(candle(i));
        }
        atr.reset();
        assert!(!atr.is_ready());
        assert_eq!(atr.value(), None);
    }

    #[test]
    fn byte_parity_with_wickra_core() {
        let mut ours = Atr::<14>::new();
        let mut theirs = wickra_core::Atr::new(14).unwrap();
        for i in 0..1000 {
            let base = 100.0 + 20.0 * (f64::from(i) * 0.08).sin();
            let (open, high, low, close) = (base, base + 3.0, base - 3.0, base + 1.5);
            let ours_c = Candle::new(open, high, low, close, 10.0, i64::from(i));
            let theirs_c =
                wickra_core::Candle::new(open, high, low, close, 10.0, i64::from(i)).unwrap();
            let a = ours.update(ours_c);
            let b = <wickra_core::Atr as wickra_core::Indicator>::update(&mut theirs, theirs_c);
            assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits), "index {i}");
        }
    }
}
