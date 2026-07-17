//! Lifecycle-contract parity: `warmup_period`, `is_ready` and `reset` behave the
//! same as the std `wickra-core`, indicator by indicator.
//!
//! Value parity lives in `parity.rs` / `golden.rs`; this file pins the *state
//! machine*: how long each indicator warms up, when `is_ready` flips, and that
//! `reset` returns it to the freshly-constructed state.

use embed_core::{Atr, Candle, Ema, Indicator, Roc, Rsi, Sma};
// Bring the std reference's trait methods (`warmup_period`) into scope for
// resolution without shadowing `embed_core::Indicator` — receivers are distinct
// types so there is no ambiguity.
use wickra_core::Indicator as _;

fn feed_scalar<I: Indicator<Input = f64>>(ind: &mut I, n: usize) {
    for i in 0..n {
        ind.update(1.0 + (i as f64) * 0.5);
    }
}

/// `warmup_period` matches `wickra-core`, `is_ready` is false before that many
/// inputs and true after, and `reset` returns to not-ready.
#[test]
fn sma_lifecycle_matches_wickra_core() {
    let mut e = Sma::<20>::new();
    let w = wickra_core::Sma::new(20).unwrap();
    assert_eq!(e.warmup_period(), w.warmup_period(), "sma warmup");
    assert!(!e.is_ready());
    let warmup = e.warmup_period();
    feed_scalar(&mut e, warmup - 1);
    assert!(!e.is_ready(), "not ready one short of warmup");
    feed_scalar(&mut e, 1);
    assert!(e.is_ready(), "ready at warmup");
    e.reset();
    assert!(!e.is_ready(), "reset clears readiness");
    assert_eq!(e.name(), "sma");
}

#[test]
fn ema_lifecycle_matches_wickra_core() {
    let mut e = Ema::new(20);
    let w = wickra_core::Ema::new(20).unwrap();
    assert_eq!(e.warmup_period(), w.warmup_period(), "ema warmup");
    let warmup = e.warmup_period();
    feed_scalar(&mut e, warmup - 1);
    assert!(!e.is_ready());
    feed_scalar(&mut e, 1);
    assert!(e.is_ready());
    e.reset();
    assert!(!e.is_ready());
}

#[test]
fn rsi_lifecycle_matches_wickra_core() {
    let mut e = Rsi::<14>::new();
    let w = wickra_core::Rsi::new(14).unwrap();
    assert_eq!(e.warmup_period(), w.warmup_period(), "rsi warmup");
    let warmup = e.warmup_period();
    feed_scalar(&mut e, warmup - 1);
    assert!(!e.is_ready());
    feed_scalar(&mut e, 1);
    assert!(e.is_ready());
    e.reset();
    assert!(!e.is_ready());
}

#[test]
fn roc_lifecycle_matches_wickra_core() {
    let mut e = Roc::<10>::new();
    let w = wickra_core::Roc::new(10).unwrap();
    assert_eq!(e.warmup_period(), w.warmup_period(), "roc warmup");
    let warmup = e.warmup_period();
    feed_scalar(&mut e, warmup - 1);
    assert!(!e.is_ready());
    feed_scalar(&mut e, 1);
    assert!(e.is_ready());
    e.reset();
    assert!(!e.is_ready());
}

#[test]
fn atr_lifecycle_matches_wickra_core() {
    let mut e = Atr::<14>::new();
    let w = wickra_core::Atr::new(14).unwrap();
    assert_eq!(e.warmup_period(), w.warmup_period(), "atr warmup");
    let warmup = e.warmup_period();
    for i in 0..warmup - 1 {
        let base = 100.0 + i as f64;
        let t = i64::try_from(i).unwrap();
        e.update(Candle::new(
            base,
            base + 2.0,
            base - 2.0,
            base + 1.0,
            10.0,
            t,
        ));
    }
    assert!(!e.is_ready(), "not ready one short of warmup");
    e.update(Candle::new(200.0, 202.0, 198.0, 201.0, 10.0, 99));
    assert!(e.is_ready(), "ready at warmup");
    e.reset();
    assert!(!e.is_ready(), "reset clears readiness");
    assert_eq!(e.name(), "atr");
}
