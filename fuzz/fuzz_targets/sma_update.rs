#![no_main]
//! Fuzz the scalar-input indicators (`Sma`, `Rsi`, `Roc`) that share the raw-f64
//! decode path. Arbitrary bytes are decoded as a stream of finite prices; feeding
//! them must never panic, every `Some` output must be finite, and `is_ready` must
//! never flip back to false once true.

use embed_core::{Indicator, Roc, Rsi, Sma};
use libfuzzer_sys::fuzz_target;

/// Decode bytes into up to 128 finite, bounded prices (8 bytes per `f64`).
fn prices(data: &[u8]) -> Vec<f64> {
    data.chunks_exact(8)
        .take(128)
        .map(|c| f64::from_bits(u64::from_le_bytes(c.try_into().unwrap())))
        .filter(|x| x.is_finite() && x.abs() < 1.0e12)
        .collect()
}

fn drive<I: Indicator<Input = f64>>(mut ind: I, stream: &[f64]) {
    let mut was_ready = false;
    for &x in stream {
        if let Some(v) = ind.update(x) {
            assert!(v.is_finite(), "non-finite output {v}");
        }
        let ready = ind.is_ready();
        if was_ready {
            assert!(ready, "is_ready flipped back to false");
        }
        was_ready = ready;
    }
}

fuzz_target!(|data: &[u8]| {
    let stream = prices(data);
    drive(Sma::<20>::new(), &stream);
    drive(Rsi::<14>::new(), &stream);
    drive(Roc::<10>::new(), &stream);
});
