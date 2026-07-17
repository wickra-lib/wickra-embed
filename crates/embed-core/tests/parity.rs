//! Live byte-parity against the std `wickra-core`, plus the `math` switch-point.
//!
//! `golden.rs` pins `embed-core` against *frozen* `golden/expected/*.csv`. This
//! file proves the complementary direction: over an independent, longer input
//! series, every `embed-core` output is **bit-for-bit** (`f64::to_bits`) equal to
//! the value the *live* `wickra-core` produces right now — so a drift introduced
//! by a `wickra-core` bump is caught even before the golden files are re-blessed.
//!
//! It also asserts the `math` switch-point directly: the `libm` implementations
//! (`fabs`, `fmax`, `fma`) the `no_std` build routes through are bit-identical to
//! the `std` methods (`f64::abs`, `f64::max`, `f64::mul_add`) the host build uses.
//! That is what makes the value on a soft-float Cortex-M0 equal the value on the
//! host.

use embed_core::{Atr, Candle, Ema, Indicator, Roc, Rsi, Sma};

/// A longer, drift-and-oscillate path than the golden vectors, to exercise the
/// rolling-sum reseed many times: `x(i) = 100 + 12·sin(i/5) + 0.03·i`.
const N: usize = 500;

fn price(i: usize) -> f64 {
    100.0 + 12.0 * (i as f64 / 5.0).sin() + 0.03 * i as f64
}

fn candle(i: usize) -> (f64, f64, f64, f64) {
    let open = price(i);
    let close = price(i + 1);
    (open, open.max(close) + 1.0, open.min(close) - 1.0, close)
}

fn assert_scalar_parity<E, W>(label: &str, mut ours: E, mut theirs: W)
where
    E: FnMut(f64) -> Option<f64>,
    W: FnMut(f64) -> Option<f64>,
{
    for i in 0..N {
        let x = price(i);
        assert_eq!(
            ours(x).map(f64::to_bits),
            theirs(x).map(f64::to_bits),
            "{label}: index {i} differs"
        );
    }
}

#[test]
fn sma_matches_live_wickra_core() {
    let mut e = Sma::<20>::new();
    let mut w = wickra_core::Sma::new(20).unwrap();
    assert_scalar_parity(
        "sma20",
        |x| e.update(x),
        |x| <wickra_core::Sma as wickra_core::Indicator>::update(&mut w, x),
    );
}

#[test]
fn ema_matches_live_wickra_core() {
    let mut e = Ema::new(20);
    let mut w = wickra_core::Ema::new(20).unwrap();
    assert_scalar_parity(
        "ema20",
        |x| e.update(x),
        |x| <wickra_core::Ema as wickra_core::Indicator>::update(&mut w, x),
    );
}

#[test]
fn rsi_matches_live_wickra_core() {
    let mut e = Rsi::<14>::new();
    let mut w = wickra_core::Rsi::new(14).unwrap();
    assert_scalar_parity(
        "rsi14",
        |x| e.update(x),
        |x| <wickra_core::Rsi as wickra_core::Indicator>::update(&mut w, x),
    );
}

#[test]
fn roc_matches_live_wickra_core() {
    let mut e = Roc::<10>::new();
    let mut w = wickra_core::Roc::new(10).unwrap();
    assert_scalar_parity(
        "roc10",
        |x| e.update(x),
        |x| <wickra_core::Roc as wickra_core::Indicator>::update(&mut w, x),
    );
}

#[test]
fn atr_matches_live_wickra_core() {
    let mut e = Atr::<14>::new();
    let mut w = wickra_core::Atr::new(14).unwrap();
    for i in 0..N {
        let (open, high, low, close) = candle(i);
        let ec = Candle::new(open, high, low, close, 1000.0, i64::try_from(i).unwrap());
        let wc =
            wickra_core::Candle::new(open, high, low, close, 1000.0, i64::try_from(i).unwrap())
                .unwrap();
        let a = e.update(ec);
        let b = <wickra_core::Atr as wickra_core::Indicator>::update(&mut w, wc);
        assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits), "atr14: index {i}");
    }
}

// --- The `math` switch-point: libm == std, bit-for-bit ----------------------

/// A spread of values covering signs, magnitudes and the fused-multiply-add
/// rounding cases the indicators actually feed through `math`.
fn math_probe_values() -> Vec<f64> {
    let mut v = vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        -0.5,
        f64::MIN_POSITIVE,
        1e-300,
        1e300,
        core::f64::consts::PI,
        -core::f64::consts::E,
    ];
    for i in 0..N {
        v.push(price(i));
        v.push(-price(i));
    }
    v
}

#[test]
fn libm_abs_equals_std_abs() {
    for &x in &math_probe_values() {
        assert_eq!(
            f64::abs(x).to_bits(),
            libm::fabs(x).to_bits(),
            "abs({x}) diverges"
        );
    }
}

#[test]
fn libm_max_equals_std_max() {
    let vals = math_probe_values();
    for &a in &vals {
        for &b in &[1.0, -1.0, 0.0, a * 0.5, 100.0] {
            assert_eq!(
                f64::max(a, b).to_bits(),
                libm::fmax(a, b).to_bits(),
                "max({a}, {b}) diverges"
            );
        }
    }
}

#[test]
fn libm_fma_equals_std_mul_add() {
    let vals = math_probe_values();
    for &a in &vals {
        let (b, c) = (0.9, a * 0.1);
        assert_eq!(
            f64::mul_add(a, b, c).to_bits(),
            libm::fma(a, b, c).to_bits(),
            "mul_add({a}, {b}, {c}) diverges"
        );
    }
}
