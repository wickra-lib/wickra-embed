# Byte-parity with wickra-core

The moat of `wickra-embed` is a single guarantee: **the value it computes on bare
metal is byte-for-byte identical to the value the std `wickra-core` computes on a
server.** This document explains why that holds and how it is verified.

## Why byte-parity, not "close enough"

A trading system that runs signals on an MCU and re-checks them on a server must
be able to compare the two results *exactly*. If the embedded value merely
approximated the server value, every comparison would need a tolerance, and a
real discrepancy could hide inside that tolerance. Bit-for-bit equality removes
the ambiguity: the values are the same, or there is a bug.

## Why it is achievable

IEEE-754 `f64` arithmetic is deterministic. `a + b`, `a - b`, `a * b`, `a / b`
and the fused `fma` each produce one correctly-rounded result that does not
depend on the platform — a soft-float Cortex-M0, a hardware-FPU Cortex-M4F, and
an x86-64 server all compute the identical bits, provided the **operations happen
in the identical order**.

So byte-parity reduces to one engineering rule: `embed-core` performs the exact
same floating-point operations, in the exact same order, as `wickra-core`. Where
the std code keeps a `Box<[f64]>` window, the embedded code keeps a const-generic
ring — but the arithmetic is preserved to the operation:

- **Sma** — the same rolling `sum -= old; sum += new` order, the same reseed of
  the running sum from the live window every `16 · N` updates (bounding drift),
  and the same final `sum / N` division.
- **Ema** — the same seed (mean of the first `N` inputs), the same precomputed
  `1 - alpha`, and the same fused `alpha.mul_add(x, one_minus_alpha * prev)`.
- **Rsi / Atr** — the same Wilder seed (mean of the first `N` gains/losses or
  true ranges) and the same reciprocal-hoisted fused smoothing.
- **Roc** — the same `(x - x_ago) / x_ago * 100`, with the same zero-guard.

The `Vec` warmup buffers of the std code are replaced by incremental running sums.
Summing `x₀, x₁, …` incrementally is the *identical fold* to `buf.iter().sum()`
(both start from `0.0` and add in arrival order), so the seed values match to the
bit — no buffer needed.

## How it is verified

Parity is not asserted by inspection; it is tested:

1. **Against the oracle.** `wickra-core` is a dev-dependency (host tests only,
   never linked into the bare-metal build). Each indicator is run alongside its
   `wickra-core` counterpart over a long, drift-exercising input series, and every
   output is compared with `f64::to_bits` — bit equality, not approximate. These
   live in the indicators' `#[cfg(test)]` modules today and expand into the
   dedicated golden harness in a later phase.
2. **Across targets.** Because the arithmetic is order-identical and IEEE-754 is
   deterministic, the same bits fall out on `thumbv7em`, `thumbv6m` and the host.
   The `math` switch point is tested to confirm the `libm` path equals the `std`
   path for `abs`, `max` and `mul_add`, so `no_std` cannot silently diverge.

Any divergence from the `wickra-core` reference is a bug, not a rounding
difference to be tolerated.

## The one rule for contributors

When adding an indicator, put the std `wickra-core/src/indicators/<name>.rs` next
to your implementation and match its operation order exactly — the rolling-sum
add/subtract order, the reseed cadence, the division and rounding points. Then
ship a byte-parity test. A subtle reordering (reseeding differently, iterating a
ring the other way) is an invisible f64-drift bug that only the parity test will
catch.
