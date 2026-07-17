//! Host sanity example for `wickra-embed`.
//!
//! Reads the committed golden price vector (`golden/inputs/prices-01.csv`), feeds
//! it through the allocation-free `Sma<20>` and `Ema(20)`, and prints each bar's
//! output once warm. This is the same code that runs on bare metal — only here it
//! is linked against `std` so it can read a file and print. Run with:
//!
//! ```text
//! cargo run --manifest-path examples/host/Cargo.toml
//! ```

use std::path::Path;

use embed_core::{Ema, Indicator, Sma};

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../golden/inputs/prices-01.csv");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {} (run from the repo): {e}", path.display()));

    let prices: Vec<f64> = text
        .lines()
        .skip(1) // header: "price"
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().parse().expect("finite f64 price"))
        .collect();

    let mut sma = Sma::<20>::new();
    let mut ema = Ema::new(20);

    println!("bar   price      sma20        ema20");
    for (i, &price) in prices.iter().enumerate() {
        let s = sma.update(price);
        let e = ema.update(price);
        // Only print once both are warm, so the columns line up with real values.
        if let (Some(s), Some(e)) = (s, e) {
            println!("{i:>3}  {price:>8.4}  {s:>10.5}  {e:>10.5}");
        }
    }

    println!(
        "\nfed {} prices; sma warmup = {}, ema warmup = {}",
        prices.len(),
        sma.warmup_period(),
        ema.warmup_period()
    );
}
