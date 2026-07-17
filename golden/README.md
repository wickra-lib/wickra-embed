# Golden vectors — the byte-parity moat

These files are the reference for `wickra-embed`'s single guarantee: the value
the `#![no_std]` `embed-core` computes on bare metal is **byte-for-byte
identical** to the value the std [`wickra-core`](https://github.com/wickra-lib/wickra)
computes on a server. See [docs/PARITY.md](../docs/PARITY.md).

> **Never edit these files by hand.** They are generated from a closed-form
> formula and from `wickra-core`, and checked bit-for-bit. A hand edit silently
> breaks the parity guarantee. Regenerate with the bless command below.

## Layout

| File | Contents |
|------|----------|
| `inputs/prices-01.csv` | 64 `f64` prices (one `price` column). |
| `inputs/ohlc-01.csv` | 64 OHLC bars (`open,high,low,close,volume,timestamp`). |
| `expected/sma20.csv` | `Sma<20>` output, one cell per input row. |
| `expected/ema20.csv` | `Ema` (period 20) output. |
| `expected/rsi14.csv` | `Rsi<14>` output. |
| `expected/atr14.csv` | `Atr<14>` output (over `ohlc-01`). |
| `expected/roc10.csv` | `Roc<10>` output. |

An **empty cell** in an `expected` column is `None` — the indicator is still
warming up. So `sma20` has 19 empty cells then a value from the 20th row on,
`rsi14` warms up for 15 rows, `atr14` for 14, and `roc10` for 11.

## The input formula

Both input vectors come from one deterministic price path (64 samples):

```
x(i) = 100 + 10·sin(i / 6) + 0.05·i        for i = 0, 1, …, 63
```

It drifts upward (the `0.05·i` term) while oscillating (the `sin`), which
exercises the rolling-sum reseed and keeps returns non-degenerate (a purely
geometric path would give constant log-returns and an undefined RSI). The OHLC
bars are derived from the same path: `open = x(i)`, `close = x(i+1)`,
`high = max(open, close) + 1`, `low = min(open, close) − 1`, so every candle is
valid. Volume is `1000 + i` and the timestamp is `i`.

Numbers are written in each value's shortest round-tripping decimal form, so
reading a cell back reconstructs the exact `f64` bits.

## Regenerating (bless)

The generator and the replay check both live in
`crates/embed-core/tests/golden.rs`. To regenerate every file from the formula
and from `wickra-core`:

```bash
cargo test -p embed-core --test golden bless -- --ignored
```

Then commit the result. The always-on `golden_replay` test feeds the committed
inputs through `embed-core` and asserts the output is bit-for-bit
(`f64::to_bits`) equal to these expected columns:

```bash
cargo test -p embed-core --test golden golden_replay
```

Regenerate only when the `wickra-core` reference itself changes (a new pinned
version) — never to "fix" a failing replay, which would be papering over a real
`f64` drift.
