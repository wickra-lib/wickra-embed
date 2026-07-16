//! Simple Moving Average — const-generic, no-alloc, byte-parity with
//! `wickra_core::Sma`.

use crate::ring::Ring;
use crate::traits::Indicator;

/// How often (in finite updates) the incremental sum is reseeded from the live
/// window, as a multiple of `N`. Must equal `wickra-core`'s constant: the drift
/// bound and, more importantly, the exact reseed *cadence* are part of the
/// byte-parity contract.
const RECOMPUTE_EVERY: usize = 16;

/// Simple Moving Average over a fixed window of `N` prices.
///
/// Keeps a rolling sum so each update is O(1); the sum is reseeded from the live
/// window every `16 · N` finite updates to bound floating-point drift, exactly
/// as `wickra_core::Sma` does. Output is `sum / N` once the window is full,
/// `None` before.
///
/// # Example
///
/// ```
/// use embed_core::{Indicator, Sma};
///
/// let mut sma = Sma::<3>::new();
/// assert_eq!(sma.update(1.0), None);
/// assert_eq!(sma.update(2.0), None);
/// assert_eq!(sma.update(3.0), Some(2.0));
/// assert_eq!(sma.update(4.0), Some(3.0));
/// ```
#[derive(Debug, Clone)]
pub struct Sma<const N: usize> {
    buf: Ring<N>,
    sum: f64,
    updates_since_recompute: usize,
}

impl<const N: usize> Sma<N> {
    /// Construct a new SMA over a window of `N`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: Ring::new(),
            sum: 0.0,
            updates_since_recompute: 0,
        }
    }

    /// Current value if the window is full.
    #[must_use]
    pub fn value(&self) -> Option<f64> {
        if self.buf.is_full() {
            Some(self.sum / N as f64)
        } else {
            None
        }
    }
}

impl<const N: usize> Default for Sma<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Indicator for Sma<N> {
    type Input = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.value();
        }
        // Subtract-then-add in the same order as wickra-core: the evicted oldest
        // value leaves the sum before the new one enters it.
        match self.buf.push(input) {
            Some(old) => {
                self.sum -= old;
                self.sum += input;
            }
            None => {
                self.sum += input;
            }
        }
        self.updates_since_recompute += 1;
        if self.updates_since_recompute >= RECOMPUTE_EVERY * N {
            self.sum = self.buf.chrono_sum();
            self.updates_since_recompute = 0;
        }
        self.value()
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn warmup_period(&self) -> usize {
        N
    }

    fn is_ready(&self) -> bool {
        self.buf.is_full()
    }

    fn name(&self) -> &'static str {
        "sma"
    }
}

#[cfg(test)]
mod tests {
    use super::Sma;
    use crate::traits::Indicator;

    #[test]
    fn warmup_then_mean() {
        let mut sma = Sma::<4>::new();
        assert!(!sma.is_ready());
        for x in [10.0, 20.0, 30.0] {
            assert_eq!(sma.update(x), None);
        }
        assert_eq!(sma.update(40.0), Some(25.0));
        assert!(sma.is_ready());
        assert_eq!(sma.warmup_period(), 4);
    }

    #[test]
    fn reset_clears_state() {
        let mut sma = Sma::<3>::new();
        for x in [1.0, 2.0, 3.0, 4.0] {
            sma.update(x);
        }
        assert!(sma.is_ready());
        sma.reset();
        assert!(!sma.is_ready());
        assert_eq!(sma.value(), None);
    }

    #[test]
    fn non_finite_is_ignored() {
        let mut sma = Sma::<2>::new();
        sma.update(2.0);
        let ready = sma.update(4.0);
        assert_eq!(ready, Some(3.0));
        // NaN leaves the value unchanged.
        assert_eq!(sma.update(f64::NAN), Some(3.0));
        assert_eq!(sma.update(f64::INFINITY), Some(3.0));
    }

    /// Byte-for-byte parity with the std `wickra_core::Sma` over a long,
    /// drift-exercising series (crosses several reseed boundaries).
    #[test]
    fn byte_parity_with_wickra_core() {
        let mut ours = Sma::<20>::new();
        let mut theirs = wickra_core::Sma::new(20).unwrap();
        for i in 0..2000 {
            let x = 100.0 + 25.0 * (f64::from(i) * 0.05).sin() + 0.01 * f64::from(i);
            let a = ours.update(x);
            let b = <wickra_core::Sma as wickra_core::Indicator>::update(&mut theirs, x);
            assert_eq!(
                a.map(f64::to_bits),
                b.map(f64::to_bits),
                "divergence at index {i}"
            );
        }
    }
}
