//! Relative Strength Index (Wilder) — const-generic, no-alloc, byte-parity with
//! `wickra_core::Rsi`.

use crate::math;
use crate::traits::Indicator;

/// Relative Strength Index over a Wilder period `N`.
///
/// Uses Wilder's smoothing (`alpha = 1 / N`). The seed averages the first `N`
/// gains and losses; from there each step is the reciprocal-hoisted fused
/// smoothing `avg.mul_add(N - 1, x) / N`, identical to `wickra_core::Rsi`. The
/// first output appears after `N + 1` inputs.
///
/// # Example
///
/// ```
/// use embed_core::{Indicator, Rsi};
///
/// let mut rsi = Rsi::<14>::new();
/// let mut last = None;
/// for i in 0..40 {
///     last = rsi.update(100.0 + f64::from(i % 5));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Rsi<const N: usize> {
    n_minus_1: f64,
    inv_period: f64,
    prev_close: f64,
    has_prev: bool,
    /// Running sums and count of the seed gains/losses, replacing wickra-core's
    /// two `Vec<f64>` seed buffers. Incremental summation in arrival order is the
    /// identical fold to `seed_buf.iter().sum()`, so the seed averages match.
    seed_count: usize,
    seed_sum_gain: f64,
    seed_sum_loss: f64,
    avg_gain: f64,
    avg_loss: f64,
    avgs_seeded: bool,
    last_value: Option<f64>,
}

impl<const N: usize> Rsi<N> {
    /// Construct a new RSI over a Wilder period of `N`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            n_minus_1: (N - 1) as f64,
            inv_period: 1.0 / N as f64,
            prev_close: 0.0,
            has_prev: false,
            seed_count: 0,
            seed_sum_gain: 0.0,
            seed_sum_loss: 0.0,
            avg_gain: 0.0,
            avg_loss: 0.0,
            avgs_seeded: false,
            last_value: None,
        }
    }

    /// Current value if available.
    #[must_use]
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }

    /// RSI from smoothed average gain/loss, collapsed to a single division.
    /// `100·ag/(ag+al)`, with the no-movement case (`ag+al == 0`) neutral at 50.
    fn rsi_from_avgs(avg_gain: f64, avg_loss: f64) -> f64 {
        let denom = avg_gain + avg_loss;
        if denom == 0.0 {
            50.0
        } else {
            100.0 * avg_gain / denom
        }
    }
}

impl<const N: usize> Default for Rsi<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Indicator for Rsi<N> {
    type Input = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_value;
        }
        if !self.has_prev {
            self.prev_close = input;
            self.has_prev = true;
            return None;
        }
        let prev = self.prev_close;
        self.prev_close = input;

        let diff = input - prev;
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };

        if self.avgs_seeded {
            let smoothed_gain =
                math::mul_add(self.avg_gain, self.n_minus_1, gain) * self.inv_period;
            let smoothed_loss =
                math::mul_add(self.avg_loss, self.n_minus_1, loss) * self.inv_period;
            self.avg_gain = smoothed_gain;
            self.avg_loss = smoothed_loss;
            let v = Self::rsi_from_avgs(smoothed_gain, smoothed_loss);
            self.last_value = Some(v);
            return Some(v);
        }

        self.seed_sum_gain += gain;
        self.seed_sum_loss += loss;
        self.seed_count += 1;
        if self.seed_count == N {
            let ag = self.seed_sum_gain / N as f64;
            let al = self.seed_sum_loss / N as f64;
            self.avg_gain = ag;
            self.avg_loss = al;
            self.avgs_seeded = true;
            let v = Self::rsi_from_avgs(ag, al);
            self.last_value = Some(v);
            return Some(v);
        }
        None
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn warmup_period(&self) -> usize {
        N + 1
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "rsi"
    }
}

#[cfg(test)]
mod tests {
    use super::Rsi;
    use crate::traits::Indicator;

    #[test]
    fn warmup_and_bounds() {
        let mut rsi = Rsi::<3>::new();
        // First input only sets the baseline.
        assert_eq!(rsi.update(100.0), None);
        assert_eq!(rsi.warmup_period(), 4);
        let mut last = None;
        for i in 1..40 {
            last = rsi.update(100.0 + f64::from(i % 7));
        }
        let v = last.unwrap();
        assert!((0.0..=100.0).contains(&v), "rsi out of range: {v}");
        assert!(rsi.is_ready());
    }

    #[test]
    fn all_gains_saturates_to_100() {
        let mut rsi = Rsi::<3>::new();
        let mut last = None;
        for i in 0..20 {
            last = rsi.update(100.0 + f64::from(i));
        }
        assert_eq!(last, Some(100.0));
    }

    #[test]
    fn byte_parity_with_wickra_core() {
        let mut ours = Rsi::<14>::new();
        let mut theirs = wickra_core::Rsi::new(14).unwrap();
        for i in 0..1000 {
            let x = 100.0 + 20.0 * (f64::from(i) * 0.09).sin() + 5.0 * (f64::from(i) * 0.3).cos();
            let a = ours.update(x);
            let b = <wickra_core::Rsi as wickra_core::Indicator>::update(&mut theirs, x);
            assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits), "index {i}");
        }
    }
}
