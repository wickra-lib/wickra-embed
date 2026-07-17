#![no_main]
//! Fuzz the candle-input `Atr`. Arbitrary bytes are decoded four-at-a-time into
//! OHLC values; only well-formed candles (`high >= max(open,close)`,
//! `low <= min(open,close)`, all finite) reach the indicator — the same
//! precondition `Candle::new` guarantees. Feeding them must never panic and every
//! emitted true-range average must be finite and non-negative.

use embed_core::{Atr, Candle, Indicator};
use libfuzzer_sys::fuzz_target;

fn finite_bounded(x: f64) -> bool {
    x.is_finite() && x.abs() < 1.0e12
}

fuzz_target!(|data: &[u8]| {
    let mut atr = Atr::<14>::new();
    let mut was_ready = false;
    // 4 f64 per candle = 32 bytes; cap at 64 candles.
    for (i, chunk) in data.chunks_exact(32).take(64).enumerate() {
        let mut vals = [0.0_f64; 4];
        for (v, bytes) in vals.iter_mut().zip(chunk.chunks_exact(8)) {
            *v = f64::from_bits(u64::from_le_bytes(bytes.try_into().unwrap()));
        }
        let [open, close, hi_pad, lo_pad] = vals;
        if !vals.iter().copied().all(finite_bounded) {
            continue;
        }
        // Build a valid candle: high above the body, low below it.
        let high = open.max(close) + hi_pad.abs();
        let low = open.min(close) - lo_pad.abs();
        if !finite_bounded(high) || !finite_bounded(low) {
            continue;
        }
        let candle = Candle::new(open, high, low, close, 1000.0, i64::try_from(i).unwrap());
        if let Some(v) = atr.update(candle) {
            assert!(v.is_finite() && v >= 0.0, "atr produced {v}");
        }
        let ready = atr.is_ready();
        if was_ready {
            assert!(ready, "atr is_ready flipped back to false");
        }
        was_ready = ready;
    }
});
