//! The single switch point for f64 operations that `core` does not provide.
//!
//! On a host (`feature = "std"`) these delegate to the standard-library float
//! methods. On bare metal (`no_std`) they delegate to `libm`. Both paths are
//! IEEE-754 identical for the operations used here — `fma` is the uniquely
//! correctly-rounded fused multiply-add, `fabs` clears the sign bit, and `fmax`
//! matches `f64::max` on finite inputs — so an indicator computes the same bits
//! whether it runs on a server or a Cortex-M0. The `parity` tests assert this
//! equality directly.
//!
//! Simple moving average, EMA and rate-of-change need only `+ - * /` and so use
//! none of these; RSI and ATR need the fused multiply-add, and ATR additionally
//! needs `abs`/`max` for the true range.

/// `|x|`.
#[cfg(feature = "std")]
#[inline]
#[must_use]
pub fn abs(x: f64) -> f64 {
    x.abs()
}

/// `|x|`.
#[cfg(not(feature = "std"))]
#[inline]
#[must_use]
pub fn abs(x: f64) -> f64 {
    libm::fabs(x)
}

/// The larger of `a` and `b` (IEEE `maximumNumber`; matches `f64::max`).
#[cfg(feature = "std")]
#[inline]
#[must_use]
pub fn max(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// The larger of `a` and `b` (IEEE `maximumNumber`; matches `f64::max`).
#[cfg(not(feature = "std"))]
#[inline]
#[must_use]
pub fn max(a: f64, b: f64) -> f64 {
    libm::fmax(a, b)
}

/// Fused multiply-add `a * b + c` with a single rounding.
#[cfg(feature = "std")]
#[inline]
#[must_use]
pub fn mul_add(a: f64, b: f64, c: f64) -> f64 {
    a.mul_add(b, c)
}

/// Fused multiply-add `a * b + c` with a single rounding.
#[cfg(not(feature = "std"))]
#[inline]
#[must_use]
pub fn mul_add(a: f64, b: f64, c: f64) -> f64 {
    libm::fma(a, b, c)
}

#[cfg(test)]
mod tests {
    use super::{abs, max, mul_add};

    #[test]
    fn abs_matches_std() {
        assert_eq!(abs(-3.5).to_bits(), 3.5_f64.to_bits());
        assert_eq!(abs(3.5).to_bits(), 3.5_f64.to_bits());
        assert_eq!(abs(0.0).to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn max_matches_std() {
        assert_eq!(max(2.0, 7.0).to_bits(), 7.0_f64.max(2.0).to_bits());
        assert_eq!(max(7.0, 2.0).to_bits(), 7.0_f64.max(2.0).to_bits());
    }

    #[test]
    fn mul_add_matches_std() {
        let (a, b, c) = (1.234_567, 8.910_11, -2.345_67);
        assert_eq!(mul_add(a, b, c).to_bits(), a.mul_add(b, c).to_bits());
    }
}
