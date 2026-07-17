# Latency

`wickra-embed` exists to run indicator math where the *worst-case* per-update cost
must be bounded — HFT co-processors and hard-real-time firmware care about the tail
far more than the mean. This document explains why the latency is bounded, how it
is measured, and what the numbers are.

## Why the latency is bounded

Every indicator in `embed-core` is **O(1) per update** with fixed-size state:

- **`Sma`** keeps a rolling sum and a const-generic ring buffer. An update
  subtracts the oldest value, adds the new one, and divides — constant work
  regardless of the window length `N` (the window only sets the buffer size).
- **`Ema`**, **`Roc`** keep a handful of scalars and do a fixed amount of
  arithmetic per tick — no buffer at all.
- **`Rsi`**, **`Atr`** keep Wilder running averages: a fixed reciprocal-hoisted
  fused smoothing step per tick.

There is no allocation on any path (the core is `#![no_std]` with no allocator
linked), no unbounded loop, and no panic on the hot path. So the per-update cost
has a hard ceiling — the property that matters when a late tick is a missed trade
or a blown deadline.

## The one worst-case tick: the rolling-sum reseed

The single source of variation is `Sma`'s rolling-sum reseed. To bound the
floating-point drift a naive `sum -= old; sum += new` accumulates, `Sma` reseeds
its running sum from the live ring buffer every `RECOMPUTE_EVERY = 16` updates.
That reseed is an `O(N)` sum over the window — but it happens once per 16 ticks,
and `N` is a small compile-time constant, so the amortised and worst-case costs
are both bounded and known. The bench measures the reseed explicitly (see below)
rather than hiding it in an average.

## How it is measured

Two numbers matter, tracked in [BENCHMARKS.md](../BENCHMARKS.md):

- **Host (ns/update).** A [criterion](https://github.com/bheisler/criterion.rs)
  bench in `crates/embed-bench` times a single `update` call for each indicator on
  x86-64, warmed past the indicator's warmup period so it measures the steady-state
  O(1) path, not the initial fill. The core is benched with its `no_std` `libm`
  math path — the exact arithmetic that runs on the target — so host nanoseconds
  track on-device cost up to clock scaling. The nightly `bench.yml` workflow tracks
  drift; the numbers are indicative, not a CI-pinned regression gate.
- **Cortex-M (cycles/update).** The bare-metal example reads the DWT cycle counter
  around a fixed batch of `update` calls on `thumbv7em-none-eabihf` under QEMU and
  reports mean cycles per update — the same code path the host bench times, on the
  target it actually ships to.
- **Allocations.** Zero, by construction. There is nothing to measure; the absence
  is the point, and CI enforces it by building without `std`/`alloc` and by an
  alloc-symbol guard that fails if `__rust_alloc` and friends appear in the
  bare-metal object.

## Measuring the reseed directly

The `reseed/sma_20_16tick_cycle` bench times a full 16-update cycle (one reseed
included) rather than a single steady-state tick, so the reseed's amortised
contribution is visible instead of averaged away. For `Sma<20>` the 16-update
cycle runs at roughly ~220 ns, i.e. ~14 ns/update amortised — the reseed stays
inside the bounded-latency budget rather than spiking outside it.

## Results (host, indicative)

Measured with `cargo bench -p embed-bench` (criterion median on x86-64 local dev,
rounded). The Cortex-M cycle column lands with the QEMU example run; see
[BENCHMARKS.md](../BENCHMARKS.md) for the live table.

| Indicator  | Host (ns/update) | Allocations |
|------------|------------------|-------------|
| `Sma<5>`   | ~14              | 0           |
| `Sma<20>`  | ~14              | 0           |
| `Sma<50>`  | ~14              | 0           |
| `Ema` (20) | ~13              | 0           |
| `Rsi<14>`  | ~17              | 0           |
| `Atr<14>`  | ~42              | 0           |
| `Roc<10>`  | ~13              | 0           |

`Sma` cost is flat across window sizes: the rolling sum is O(1) regardless of `N`,
which only sets the ring-buffer length. `Atr` is the most expensive because it
derives a true range from an OHLC bar (two `max`/`min` and a subtraction) before
the Wilder step.

## Reproducing

```bash
# Host, per-update criterion bench:
cargo bench -p embed-bench

# MCU cycle counts (requires qemu-system-arm):
cargo run -p wickra-embed-example --release --target thumbv7em-none-eabihf
```

All figures are steady-state, post-warmup, single-update costs. Because every
update is O(1) with a bounded reseed, the worst-case per-update latency is
bounded — the property that matters for HFT and hard-real-time firmware, more than
the mean.

## See also

- [BENCHMARKS.md](../BENCHMARKS.md) — the tracked numbers and reproduction steps.
- [NO_STD.md](NO_STD.md) — the no-alloc design that makes the ceiling hold.
- [C_ABI.md](C_ABI.md) — the bounded latency the firmware C ABI inherits.
