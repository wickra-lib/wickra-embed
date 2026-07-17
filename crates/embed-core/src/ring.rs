//! A fixed-capacity ring buffer of `f64`, allocation-free.
//!
//! This is the no-alloc replacement for the `Box<[f64]>` window `wickra-core`'s
//! SMA uses. It preserves the two properties byte-parity depends on: [`push`]
//! returns the evicted oldest value in the same order the std code subtracts it
//! from the rolling sum, and [`chrono_sum`] folds oldest-to-newest exactly as
//! the std reseed does.
//!
//! [`push`]: Ring::push
//! [`chrono_sum`]: Ring::chrono_sum

/// A ring buffer holding up to `N` `f64` values.
#[derive(Debug, Clone)]
pub struct Ring<const N: usize> {
    buf: [f64; N],
    /// Index of the next slot to write — also the oldest element once full.
    head: usize,
    /// Number of slots filled, saturating at `N`.
    count: usize,
}

impl<const N: usize> Ring<N> {
    /// A new, empty ring.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0.0; N],
            head: 0,
            count: 0,
        }
    }

    /// Whether the ring holds `N` values.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.count == N
    }

    /// Number of values currently held.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Whether the ring holds no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Push `x`, overwriting the oldest slot once full.
    ///
    /// Returns the evicted oldest value if the ring was already full, or `None`
    /// while still filling. The caller (SMA) subtracts the returned value from
    /// its rolling sum *before* adding `x`, matching the std subtract-then-add
    /// order.
    pub fn push(&mut self, x: f64) -> Option<f64> {
        let evicted = if self.count == N {
            Some(self.buf[self.head])
        } else {
            self.count += 1;
            None
        };
        self.buf[self.head] = x;
        self.head += 1;
        if self.head == N {
            self.head = 0;
        }
        evicted
    }

    /// Sum the held values oldest-to-newest.
    ///
    /// When full this walks `buf[head..]` then `buf[..head]` — the exact
    /// chronological order (and thus the exact fold, starting from `0.0`) that
    /// `wickra-core` reseeds its rolling sum with, so the reseed is bit-for-bit
    /// identical.
    #[must_use]
    pub fn chrono_sum(&self) -> f64 {
        let mut sum = 0.0;
        let mut idx = self.head;
        let mut seen = 0;
        while seen < self.count {
            sum += self.buf[idx];
            idx += 1;
            if idx == N {
                idx = 0;
            }
            seen += 1;
        }
        sum
    }
}

impl<const N: usize> Default for Ring<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Ring;

    #[test]
    fn fills_then_evicts_oldest() {
        let mut ring = Ring::<3>::new();
        assert_eq!(ring.push(1.0), None);
        assert_eq!(ring.push(2.0), None);
        assert!(!ring.is_full());
        assert_eq!(ring.push(3.0), None);
        assert!(ring.is_full());
        // Now full: pushing evicts the oldest (1.0, then 2.0, ...).
        assert_eq!(ring.push(4.0), Some(1.0));
        assert_eq!(ring.push(5.0), Some(2.0));
    }

    #[test]
    fn chrono_sum_is_oldest_to_newest_after_wraparound() {
        let mut ring = Ring::<3>::new();
        for x in [1.0, 2.0, 3.0, 4.0, 5.0] {
            ring.push(x);
        }
        // Holds 3,4,5; chronological fold 0.0 + 3 + 4 + 5.
        assert_eq!(ring.chrono_sum().to_bits(), (3.0_f64 + 4.0 + 5.0).to_bits());
        assert_eq!(ring.len(), 3);
    }
}
