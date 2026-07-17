# Benchmarks

`wickra-embed` exists to run indicator math where latency is bounded and there is
no heap. Two numbers matter: the **per-update cost on a host** (criterion, in
nanoseconds) and the **per-update cost on the target MCU** (in cycles). Both are
tracked here.

## Methodology

- **Host.** A criterion bench in `crates/embed-bench` times a single `update`
  call for each indicator on x86-64, warmed past the indicator's warmup period so
  the steady-state O(1) path is measured (not the initial fill).
- **MCU.** The Cortex-M example reads the DWT cycle counter around a fixed batch
  of `update` calls on `thumbv7em-none-eabihf` under QEMU, and reports mean
  cycles per update. This measures the same code path the host bench does, on the
  soft/hard-float target it actually ships to.
- **Allocation.** Zero, by construction — the core is `#![no_std]` with no
  allocator linked. There is nothing to measure; the absence is the point, and CI
  enforces it by building without `std`/`alloc`.

## Results

Numbers land as the bench crate (phase P5) and the Cortex-M example (phase P6) are
built out. The host column below is measured (`cargo bench -p embed-bench`,
criterion median on x86-64 local dev, rounded — indicative, not a CI-pinned
regression gate; the nightly `bench.yml` tracks drift). The Cortex-M column lands
with the QEMU example (phase P6).

| Indicator  | Host (ns/update) | Cortex-M4F (cycles/update) | Allocations |
|------------|------------------|----------------------------|-------------|
| `Sma<5>`   | ~14              | _pending_                  | 0           |
| `Sma<20>`  | ~14              | _pending_                  | 0           |
| `Sma<50>`  | ~14              | _pending_                  | 0           |
| `Ema` (20) | ~13              | _pending_                  | 0           |
| `Rsi<14>`  | ~17              | _pending_                  | 0           |
| `Atr<14>`  | ~42              | _pending_                  | 0           |
| `Roc<10>`  | ~13              | _pending_                  | 0           |

`Sma` cost is flat across window sizes (5/20/50): the rolling sum is O(1)
regardless of window, which only sets the ring-buffer length. The worst-case tick
is the rolling-sum reseed every `RECOMPUTE_EVERY = 16` updates; the
`reseed/sma_20_16tick_cycle` bench times a full 16-update cycle (one reseed
included) at ~220 ns, i.e. ~14 ns/update amortised — the reseed stays inside the
bounded-latency budget.

## Reproducing

```bash
# Host, per-update criterion bench:
cargo bench -p embed-bench

# MCU cycle counts (requires qemu-system-arm):
cargo run -p embed-cortex-m-example --release --target thumbv7em-none-eabihf
```

All figures are steady-state, post-warmup, single-update costs. Because every
update is O(1) with a bounded reseed, the worst-case per-update latency is
bounded — the property that matters for HFT and hard-real-time firmware, more than
the mean.
