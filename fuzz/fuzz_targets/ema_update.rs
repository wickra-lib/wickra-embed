#![no_main]
//! Fuzz `Ema` on its own: it carries a running-average state that (unlike the
//! windowed indicators) never resets, so it is the one most sensitive to a
//! pathological input drifting the accumulator. Arbitrary bytes become a stream
//! of finite prices; updating must never panic and every emitted value must be
//! finite.

use embed_core::{Ema, Indicator};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut ema = Ema::new(20);
    let mut was_ready = false;
    for chunk in data.chunks_exact(8).take(128) {
        let x = f64::from_bits(u64::from_le_bytes(chunk.try_into().unwrap()));
        if !x.is_finite() || x.abs() >= 1.0e12 {
            continue;
        }
        if let Some(v) = ema.update(x) {
            assert!(v.is_finite(), "ema produced non-finite {v}");
        }
        let ready = ema.is_ready();
        if was_ready {
            assert!(ready, "ema is_ready flipped back to false");
        }
        was_ready = ready;
    }
});
