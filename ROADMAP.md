# Roadmap

`wickra-embed` grows in phases. Each phase lands green — formatted, linted, the
no_std core cross-built on both Cortex-M targets, and every indicator gated on a
byte-parity test against `wickra-core` — before the next begins. Dates are
deliberately absent; the order is the commitment.

## v0.1 — the allocation-free distillation

The first line establishes the moat: a small, verified, no-alloc subset of the
Wickra indicator catalogue that is byte-for-byte identical to `wickra-core`.

- **Core (`embed-core`).** The `Indicator` contract, a const-generic
  fixed-capacity ring buffer, and the verified no-alloc subset: `Sma`, `Ema`,
  `Rsi`, `Atr`, `Roc`. `#![no_std]`, `#![forbid(unsafe_code)]`, zero allocation,
  O(1) bounded-latency updates.
- **Parity oracle.** `wickra-core` as a dev-dependency only; a `parity.rs` suite
  proving each indicator's output matches byte-for-byte.
- **C ABI (`bindings/c`).** A no-alloc, stack-handle surface with a
  cbindgen-generated header, for firmware in C/C++.
- **Examples.** A host CSV runner, a C usage sample, and a `#![no_std] #![no_main]`
  Cortex-M demo runnable under QEMU.
- **Assurance.** Property tests, a fuzz target over the update path, a criterion
  bench (host ns / MCU cycles), and CI that cross-builds `thumbv7em-none-eabihf`
  and `thumbv6m-none-eabi` on every PR.

### Weg B, and why

For v0.1 the no-alloc core (`embed-core`) **reimplements** the indicator subset
against fixed-capacity storage rather than depending on `wickra-core` at runtime.
`wickra-core` is std — it uses `Box`/`Vec`/`alloc` — and cannot be linked into a
bare-metal, no-allocator build. So it stays a **dev-dependency parity oracle**:
the reference the reimplementation is proven byte-identical against, never a
runtime dependency. Preserving the exact f64 operation order of the reference
(rolling-sum add/subtract order, the periodic reseed, the final `sum / period`
division) is what makes the byte-parity hold. This is the mandatory approach for
v0.1.

## Beyond v0.1 (candidate, not committed)

- **Grow the subset.** Bollinger, MACD, Stochastic and further indicators, each
  admitted only once its no-alloc form passes byte-parity.
- **Weg A — shared no_std core upstream.** A longer-term option: make the relevant
  parts of `wickra-core` themselves `no_std` + optional-`alloc`, so `embed-core`
  could depend on them directly instead of reimplementing. This is an upstream
  refactor of `wickra-core`, tracked as future work, and explicitly **not** part
  of v0.1.
- **Fixed-point variants.** Optional `i32`/`i64` fixed-point indicators for FPGA
  soft-cores and MCUs without an FPU.
- **Host-side convenience bindings.** A thin Python/Node wrapper for offline
  validation against the same values — a convenience, not the embedded moat.

The guiding rule does not change: nothing ships that has not been proven
byte-for-byte identical to `wickra-core`, allocation-free, and panic-free on the
bare-metal targets.
