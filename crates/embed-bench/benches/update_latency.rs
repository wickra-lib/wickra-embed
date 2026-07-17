//! Per-update latency of the allocation-free `embed-core` indicators.
//!
//! Every indicator is `O(1)` per tick with a fixed-size state, so the interesting
//! numbers are (1) the steady-state cost of a single `update` and (2) the
//! worst-case tick where the windowed `Sma` reseeds its rolling sum from the live
//! ring buffer (`RECOMPUTE_EVERY = 16`). The reseed is measured separately as a
//! full 16-update cycle so its amortised contribution is visible.
//!
//! The core is benched with its `no_std` math path (default-features off at the
//! workspace edge), i.e. `libm` — the exact arithmetic that runs on the target,
//! so the host nanoseconds track the on-device cost up to clock scaling.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use embed_core::{Atr, Candle, Ema, Indicator, Roc, Rsi, Sma};

/// A finite, drifting-and-oscillating price path — never NaN/inf, and varied
/// enough that `Rsi`/`Roc` returns stay non-degenerate.
fn price(i: usize) -> f64 {
    100.0 + 12.0 * ((i as f64) * 0.1).sin() + 0.03 * i as f64
}

fn candle(i: usize) -> Candle {
    let open = price(i);
    let close = price(i + 1);
    Candle::new(
        open,
        open.max(close) + 1.0,
        open.min(close) - 1.0,
        close,
        1000.0,
        i64::try_from(i).unwrap(),
    )
}

/// Prime an indicator past its warmup so `update` measures the steady state.
fn warm_scalar<I: Indicator<Input = f64>>(ind: &mut I) {
    for i in 0..=ind.warmup_period() {
        ind.update(price(i));
    }
}

fn scalar_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_update");

    macro_rules! bench_scalar {
        ($name:literal, $ctor:expr) => {{
            let mut ind = $ctor;
            warm_scalar(&mut ind);
            let mut i = ind.warmup_period() + 1;
            group.bench_function($name, |b| {
                b.iter(|| {
                    i = i.wrapping_add(1);
                    black_box(ind.update(black_box(price(i))))
                });
            });
        }};
    }

    bench_scalar!("sma_5", Sma::<5>::new());
    bench_scalar!("sma_20", Sma::<20>::new());
    bench_scalar!("sma_50", Sma::<50>::new());
    bench_scalar!("ema_20", Ema::new(20));
    bench_scalar!("rsi_14", Rsi::<14>::new());
    bench_scalar!("roc_10", Roc::<10>::new());

    group.finish();
}

fn candle_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("candle_update");
    let mut atr = Atr::<14>::new();
    for i in 0..=atr.warmup_period() {
        atr.update(candle(i));
    }
    let mut i = atr.warmup_period() + 1;
    group.bench_function("atr_14", |b| {
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(atr.update(black_box(candle(i))))
        });
    });
    group.finish();
}

/// Worst-case: a full 16-update cycle of `Sma<20>`, which contains exactly one
/// rolling-sum reseed. Dividing the reported time by 16 and comparing to the
/// steady-state `sma_20` figure isolates the reseed's amortised cost.
fn reseed_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("reseed");
    let mut sma = Sma::<20>::new();
    warm_scalar(&mut sma);
    let mut i = sma.warmup_period() + 1;
    group.bench_function("sma_20_16tick_cycle", |b| {
        b.iter(|| {
            for _ in 0..16 {
                i = i.wrapping_add(1);
                black_box(sma.update(black_box(price(i))));
            }
        });
    });
    group.finish();
}

criterion_group!(benches, scalar_update, candle_update, reseed_cycle);
criterion_main!(benches);
