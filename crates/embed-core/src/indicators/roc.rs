//! Rate of Change — const-generic, no-alloc, byte-parity with `wickra_core::Roc`.

use crate::traits::Indicator;

/// Rate of Change as a percentage: `(close - close[N ago]) / close[N ago] · 100`,
/// where `N` is the lookback period.
///
/// A ring of exactly `N` slots holds the last `N` values; the slot about to be
/// overwritten is the value from `N` steps ago, so no `N + 1` buffer is needed.
/// Non-finite inputs are ignored (the last value is returned, state untouched),
/// matching `wickra_core::Roc`. The first output appears after `N + 1` inputs.
///
/// # Example
///
/// ```
/// use embed_core::{Indicator, Roc};
///
/// // ROC(3): prev = 100 three steps back, now = 110 -> 10%.
/// let mut roc = Roc::<3>::new();
/// assert_eq!(roc.update(100.0), None);
/// assert_eq!(roc.update(105.0), None);
/// assert_eq!(roc.update(108.0), None);
/// assert_eq!(roc.update(110.0), Some(10.0));
/// ```
#[derive(Debug, Clone)]
pub struct Roc<const N: usize> {
    buf: [f64; N],
    head: usize,
    count: usize,
    last: Option<f64>,
}

impl<const N: usize> Roc<N> {
    /// Construct a new ROC over a lookback of `N`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0.0; N],
            head: 0,
            count: 0,
            last: None,
        }
    }

    /// Current value if available.
    #[must_use]
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl<const N: usize> Default for Roc<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Indicator for Roc<N> {
    type Input = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last;
        }
        if self.count == N {
            let prev = self.buf[self.head];
            self.buf[self.head] = input;
            self.head += 1;
            if self.head == N {
                self.head = 0;
            }
            let roc = if prev == 0.0 {
                0.0
            } else {
                (input - prev) / prev * 100.0
            };
            self.last = Some(roc);
            Some(roc)
        } else {
            self.buf[self.head] = input;
            self.count += 1;
            self.head += 1;
            if self.head == N {
                self.head = 0;
            }
            None
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn warmup_period(&self) -> usize {
        N + 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "roc"
    }
}

#[cfg(test)]
mod tests {
    use super::Roc;
    use crate::traits::Indicator;

    #[test]
    fn known_value() {
        let mut roc = Roc::<3>::new();
        assert_eq!(roc.update(100.0), None);
        assert_eq!(roc.update(105.0), None);
        assert_eq!(roc.update(108.0), None);
        assert_eq!(roc.update(110.0), Some(10.0));
        assert_eq!(roc.warmup_period(), 4);
    }

    #[test]
    fn constant_series_is_zero() {
        let mut roc = Roc::<5>::new();
        let mut last = None;
        for _ in 0..20 {
            last = roc.update(42.0);
        }
        assert_eq!(last, Some(0.0));
    }

    #[test]
    fn non_finite_is_ignored() {
        let mut roc = Roc::<2>::new();
        roc.update(100.0);
        roc.update(110.0);
        let v = roc.update(120.0);
        assert!(v.is_some());
        assert_eq!(roc.update(f64::NAN), v);
    }

    #[test]
    fn byte_parity_with_wickra_core() {
        let mut ours = Roc::<12>::new();
        let mut theirs = wickra_core::Roc::new(12).unwrap();
        for i in 0..1000 {
            let x = 100.0 + 30.0 * (f64::from(i) * 0.06).sin() + 0.03 * f64::from(i);
            let a = ours.update(x);
            let b = <wickra_core::Roc as wickra_core::Indicator>::update(&mut theirs, x);
            assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits), "index {i}");
        }
    }
}
