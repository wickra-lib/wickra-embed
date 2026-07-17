//! Golden harness — the byte-parity moat, "generate once, replay forever".
//!
//! `golden/inputs/*.csv` are fixed, deterministic input vectors (a documented
//! closed-form formula, see `golden/README.md`). `golden/expected/*.csv` are the
//! outputs the **std `wickra-core`** produces for those inputs, blessed once and
//! committed byte-for-byte.
//!
//! Two entry points:
//!
//! - [`bless`] (`#[ignore]`) regenerates every `golden/` file from the formula
//!   and from `wickra-core`. Run it deliberately when the reference changes:
//!   `cargo test -p embed-core --test golden bless -- --ignored`.
//! - [`golden_replay`] (the always-on test) feeds the committed inputs through
//!   the `#![no_std]` `embed-core` indicators and asserts the output is
//!   **bit-for-bit** (`f64::to_bits`) equal to the committed expected files.
//!
//! The live-`wickra-core` direction and the `libm` math path are covered
//! separately in `parity.rs`.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use embed_core::{Atr, Candle, Ema, Indicator, Roc, Rsi, Sma};

// --- Deterministic input formula (documented in golden/README.md) -----------

/// Number of samples in every input vector.
const N: usize = 64;

/// `x(i) = 100 + 10·sin(i/6) + 0.05·i` — a drifting, oscillating price path that
/// exercises the rolling-sum reseed and keeps returns non-degenerate.
fn price(i: usize) -> f64 {
    100.0 + 10.0 * (i as f64 / 6.0).sin() + 0.05 * i as f64
}

fn price_series() -> Vec<f64> {
    (0..N).map(price).collect()
}

/// OHLC bars derived from the same path: open at `x(i)`, close at `x(i+1)`, high
/// one unit above the pair and low one unit below, so every candle is valid
/// (`high ≥ open, close ≥ low`). Volume and timestamp are incremental.
fn candle(i: usize) -> Candle {
    let open = price(i);
    let close = price(i + 1);
    let high = open.max(close) + 1.0;
    let low = open.min(close) - 1.0;
    Candle::new(
        open,
        high,
        low,
        close,
        1000.0 + i as f64,
        i64::try_from(i).unwrap(),
    )
}

fn candle_series() -> Vec<Candle> {
    (0..N).map(candle).collect()
}

// --- Paths ------------------------------------------------------------------

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../golden")
}

// --- CSV helpers ------------------------------------------------------------

/// Format an `Option<f64>` cell: a value round-trips through its shortest
/// representation; `None` (warmup) is an empty cell.
fn cell(v: Option<f64>) -> String {
    v.map_or(String::new(), |x| format!("{x}"))
}

fn parse_cell(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        Some(
            s.parse()
                .expect("expected a finite f64 in an expected cell"),
        )
    }
}

fn write_column(name: &str, header: &str, values: &[Option<f64>]) {
    let mut out = String::from(header);
    out.push('\n');
    for v in values {
        out.push_str(&cell(*v));
        out.push('\n');
    }
    let path = golden_dir().join("expected").join(name);
    fs::write(&path, out).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

fn read_column(name: &str) -> Vec<Option<f64>> {
    let path = golden_dir().join("expected").join(name);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {} (run the bless test first): {e}", path.display()));
    text.lines().skip(1).map(parse_cell).collect()
}

// --- Reference (wickra-core) column producers -------------------------------

fn ref_sma20(prices: &[f64]) -> Vec<Option<f64>> {
    let mut r = wickra_core::Sma::new(20).unwrap();
    prices
        .iter()
        .map(|&x| <wickra_core::Sma as wickra_core::Indicator>::update(&mut r, x))
        .collect()
}

fn ref_ema20(prices: &[f64]) -> Vec<Option<f64>> {
    let mut r = wickra_core::Ema::new(20).unwrap();
    prices
        .iter()
        .map(|&x| <wickra_core::Ema as wickra_core::Indicator>::update(&mut r, x))
        .collect()
}

fn ref_rsi14(prices: &[f64]) -> Vec<Option<f64>> {
    let mut r = wickra_core::Rsi::new(14).unwrap();
    prices
        .iter()
        .map(|&x| <wickra_core::Rsi as wickra_core::Indicator>::update(&mut r, x))
        .collect()
}

fn ref_roc10(prices: &[f64]) -> Vec<Option<f64>> {
    let mut r = wickra_core::Roc::new(10).unwrap();
    prices
        .iter()
        .map(|&x| <wickra_core::Roc as wickra_core::Indicator>::update(&mut r, x))
        .collect()
}

fn ref_atr14(candles: &[Candle]) -> Vec<Option<f64>> {
    let mut r = wickra_core::Atr::new(14).unwrap();
    candles
        .iter()
        .map(|c| {
            let wc =
                wickra_core::Candle::new(c.open, c.high, c.low, c.close, c.volume, c.timestamp)
                    .unwrap();
            <wickra_core::Atr as wickra_core::Indicator>::update(&mut r, wc)
        })
        .collect()
}

// --- embed-core column producers (the subject under test) -------------------

fn emb_sma20(prices: &[f64]) -> Vec<Option<f64>> {
    let mut s = Sma::<20>::new();
    prices.iter().map(|&x| s.update(x)).collect()
}

fn emb_ema20(prices: &[f64]) -> Vec<Option<f64>> {
    let mut s = Ema::new(20);
    prices.iter().map(|&x| s.update(x)).collect()
}

fn emb_rsi14(prices: &[f64]) -> Vec<Option<f64>> {
    let mut s = Rsi::<14>::new();
    prices.iter().map(|&x| s.update(x)).collect()
}

fn emb_roc10(prices: &[f64]) -> Vec<Option<f64>> {
    let mut s = Roc::<10>::new();
    prices.iter().map(|&x| s.update(x)).collect()
}

fn emb_atr14(candles: &[Candle]) -> Vec<Option<f64>> {
    let mut s = Atr::<14>::new();
    candles.iter().map(|&c| s.update(c)).collect()
}

// --- Bless ------------------------------------------------------------------

/// Regenerate every `golden/` file from the formula and from `wickra-core`.
/// Ignored by default; run explicitly when the reference changes.
#[test]
#[ignore = "bless mode: run explicitly to regenerate golden/ from wickra-core"]
fn bless() {
    let prices = price_series();
    let candles = candle_series();

    // inputs/prices-01.csv
    let mut p = String::from("price\n");
    for &x in &prices {
        writeln!(p, "{x}").unwrap();
    }
    fs::write(golden_dir().join("inputs/prices-01.csv"), p).unwrap();

    // inputs/ohlc-01.csv
    let mut o = String::from("open,high,low,close,volume,timestamp\n");
    for c in &candles {
        writeln!(
            o,
            "{},{},{},{},{},{}",
            c.open, c.high, c.low, c.close, c.volume, c.timestamp
        )
        .unwrap();
    }
    fs::write(golden_dir().join("inputs/ohlc-01.csv"), o).unwrap();

    // expected columns (from wickra-core)
    write_column("sma20.csv", "sma20", &ref_sma20(&prices));
    write_column("ema20.csv", "ema20", &ref_ema20(&prices));
    write_column("rsi14.csv", "rsi14", &ref_rsi14(&prices));
    write_column("atr14.csv", "atr14", &ref_atr14(&candles));
    write_column("roc10.csv", "roc10", &ref_roc10(&prices));

    println!("blessed golden/ with {N} samples from wickra-core");
}

// --- Replay (the always-on parity assertion) --------------------------------

/// Read the committed inputs, run the `embed-core` indicators, and assert every
/// output bit-matches the committed `wickra-core` expected column.
fn read_prices() -> Vec<f64> {
    let path = golden_dir().join("inputs/prices-01.csv");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    text.lines()
        .skip(1)
        .map(|l| l.trim().parse().expect("price f64"))
        .collect()
}

fn read_candles() -> Vec<Candle> {
    let path = golden_dir().join("inputs/ohlc-01.csv");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    text.lines()
        .skip(1)
        .map(|l| {
            let f: Vec<f64> = l
                .split(',')
                .take(5)
                .map(|s| s.trim().parse().unwrap())
                .collect();
            let t: i64 = l.split(',').nth(5).unwrap().trim().parse().unwrap();
            Candle::new(f[0], f[1], f[2], f[3], f[4], t)
        })
        .collect()
}

fn assert_bit_equal(name: &str, got: &[Option<f64>], expected: &[Option<f64>]) {
    assert_eq!(got.len(), expected.len(), "{name}: length mismatch");
    for (i, (g, e)) in got.iter().zip(expected).enumerate() {
        assert_eq!(
            g.map(f64::to_bits),
            e.map(f64::to_bits),
            "{name}: row {i} differs (embed-core {g:?} vs wickra-core {e:?})"
        );
    }
}

#[test]
fn golden_replay() {
    let prices = read_prices();
    let candles = read_candles();

    assert_bit_equal("sma20", &emb_sma20(&prices), &read_column("sma20.csv"));
    assert_bit_equal("ema20", &emb_ema20(&prices), &read_column("ema20.csv"));
    assert_bit_equal("rsi14", &emb_rsi14(&prices), &read_column("rsi14.csv"));
    assert_bit_equal("atr14", &emb_atr14(&candles), &read_column("atr14.csv"));
    assert_bit_equal("roc10", &emb_roc10(&prices), &read_column("roc10.csv"));
}
