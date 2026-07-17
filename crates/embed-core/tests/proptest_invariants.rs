//! Property invariants: whatever finite stream you feed an indicator, it must not
//! panic, its `Some` outputs stay finite, and `is_ready` never flips back to
//! false once it has turned true.
//!
//! These hold for the whole subset by construction (no `unwrap`/indexing panic,
//! warmup expressed as `None`, incremental state that never grows) — the
//! properties pin that so a future change can't quietly break it.

use embed_core::{Atr, Candle, Ema, Indicator, Roc, Rsi, Sma};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

/// Finite, sanely-bounded prices (no NaN/inf; the core's contract assumes finite
/// input, and enormous magnitudes would overflow the reference, not the core).
fn prices() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(-1.0e6..1.0e6_f64, 1..300)
}

/// Drive a scalar indicator over a stream and assert the invariants.
fn check_scalar<I: Indicator<Input = f64>>(
    mut ind: I,
    stream: &[f64],
) -> Result<(), TestCaseError> {
    let mut was_ready = false;
    for &x in stream {
        let out = ind.update(x);
        if let Some(v) = out {
            prop_assert!(v.is_finite(), "produced a non-finite value {v}");
        }
        let ready = ind.is_ready();
        if was_ready {
            prop_assert!(ready, "is_ready flipped back to false");
        }
        was_ready = ready;
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn sma_invariants(stream in prices()) {
        check_scalar(Sma::<20>::new(), &stream)?;
    }

    #[test]
    fn ema_invariants(stream in prices()) {
        check_scalar(Ema::new(20), &stream)?;
    }

    #[test]
    fn rsi_invariants(stream in prices()) {
        check_scalar(Rsi::<14>::new(), &stream)?;
    }

    #[test]
    fn roc_invariants(stream in prices()) {
        check_scalar(Roc::<10>::new(), &stream)?;
    }

    #[test]
    fn atr_invariants(stream in prices()) {
        let mut atr = Atr::<14>::new();
        let mut was_ready = false;
        for (i, &mid) in stream.iter().enumerate() {
            // Build a valid candle around each price: high above, low below.
            let candle = Candle::new(
                mid,
                mid + 2.0,
                mid - 2.0,
                mid + 1.0,
                1000.0,
                i64::try_from(i).unwrap(),
            );
            if let Some(v) = atr.update(candle) {
                prop_assert!(v.is_finite() && v >= 0.0, "atr produced {v}");
            }
            let ready = atr.is_ready();
            if was_ready {
                prop_assert!(ready, "atr is_ready flipped back to false");
            }
            was_ready = ready;
        }
    }
}
