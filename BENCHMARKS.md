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
built out. Until then this table is a placeholder describing what will be
reported, not measured figures.

| Indicator | Host (ns/update) | Cortex-M4F (cycles/update) | Allocations |
|-----------|------------------|----------------------------|-------------|
| `Sma`     | _pending_        | _pending_                  | 0           |
| `Ema`     | _pending_        | _pending_                  | 0           |
| `Rsi`     | _pending_        | _pending_                  | 0           |
| `Atr`     | _pending_        | _pending_                  | 0           |
| `Roc`     | _pending_        | _pending_                  | 0           |

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
