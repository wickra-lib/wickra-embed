//! Exponential Moving Average — no-alloc, byte-parity with `wickra_core::Ema`.

use crate::math;
use crate::traits::Indicator;

/// Exponential Moving Average with smoothing factor `alpha = 2 / (period + 1)`.
///
/// The first value is the simple mean of the first `period` inputs (the TA-Lib
/// convention); each subsequent input contributes
/// `alpha * input + (1 - alpha) * previous`, computed with the same fused
/// multiply-add and precomputed `1 - alpha` as `wickra_core::Ema`, so the
/// steady-state output is bit-for-bit identical.
///
/// The `period` is a runtime value: EMA needs no window buffer (only a running
/// warmup sum), so it stays allocation-free without a const generic.
///
/// # Example
///
/// ```
/// use embed_core::{Ema, Indicator};
///
/// let mut ema = Ema::new(3);
/// let mut last = None;
/// for i in 0..10 {
///     last = ema.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Ema {
    period: usize,
    alpha: f64,
    /// `1 - alpha`, precomputed so the recurrence avoids a subtraction per tick
    /// — and, crucially, matches wickra-core's operand order to the bit.
    one_minus_alpha: f64,
    current: f64,
    seeded: bool,
    /// Running sum and count of the warmup inputs, replacing wickra-core's
    /// `Vec<f64>` warmup buffer. Summing incrementally in arrival order is the
    /// identical fold to `warmup_buf.iter().sum()`, so the seed is byte-equal.
    warm_sum: f64,
    warm_count: usize,
}

impl Ema {
    /// Construct an EMA with the given period. A `period` of `0` yields an alpha
    /// of `2.0` and never produces a meaningful value; callers pass a positive
    /// period (the bindings validate up front).
    #[must_use]
    pub fn new(period: usize) -> Self {
        let alpha = 2.0 / (period as f64 + 1.0);
        Self {
            period,
            alpha,
            one_minus_alpha: 1.0 - alpha,
            current: 0.0,
            seeded: false,
            warm_sum: 0.0,
            warm_count: 0,
        }
    }

    /// Configured period.
    #[must_use]
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Smoothing factor `alpha`.
    #[must_use]
    pub const fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Current value if seeded.
    #[must_use]
    pub const fn value(&self) -> Option<f64> {
        if self.seeded {
            Some(self.current)
        } else {
            None
        }
    }
}

impl Indicator for Ema {
    type Input = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.value();
        }
        if self.seeded {
            let new = math::mul_add(self.alpha, input, self.one_minus_alpha * self.current);
            self.current = new;
            return Some(new);
        }
        self.warm_sum += input;
        self.warm_count += 1;
        if self.warm_count == self.period {
            let seed = self.warm_sum / self.period as f64;
            self.current = seed;
            self.seeded = true;
            return Some(seed);
        }
        None
    }

    fn reset(&mut self) {
        self.current = 0.0;
        self.seeded = false;
        self.warm_sum = 0.0;
        self.warm_count = 0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.seeded
    }

    fn name(&self) -> &'static str {
        "ema"
    }
}

#[cfg(test)]
mod tests {
    use super::Ema;
    use crate::traits::Indicator;

    #[test]
    fn alpha_and_seed() {
        let mut ema = Ema::new(3);
        assert_eq!(ema.alpha().to_bits(), (2.0_f64 / 4.0).to_bits());
        // Seed is the mean of the first three inputs.
        assert_eq!(ema.update(1.0), None);
        assert_eq!(ema.update(2.0), None);
        assert_eq!(ema.update(3.0), Some(2.0));
        assert!(ema.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut ema = Ema::new(4);
        for i in 0..10 {
            ema.update(f64::from(i));
        }
        ema.reset();
        assert!(!ema.is_ready());
        assert_eq!(ema.value(), None);
    }

    #[test]
    fn byte_parity_with_wickra_core() {
        let mut ours = Ema::new(12);
        let mut theirs = wickra_core::Ema::new(12).unwrap();
        for i in 0..1000 {
            let x = 100.0 + 15.0 * (f64::from(i) * 0.07).sin() + 0.02 * f64::from(i);
            let a = ours.update(x);
            let b = <wickra_core::Ema as wickra_core::Indicator>::update(&mut theirs, x);
            assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits), "index {i}");
        }
    }
}
