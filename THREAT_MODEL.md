# Threat Model

This document inventories what `wickra-embed` protects, where its trust boundaries
lie, and what is deliberately out of scope. It complements [SECURITY.md](SECURITY.md)
(how to report) and [ARCHITECTURE.md](ARCHITECTURE.md) (how it is built).

## What this is

`wickra-embed` is an allocation-free, `#![no_std]` indicator engine that runs on
bare metal. It reads numbers (prices, candles), folds them through streaming
indicators, and returns a number per update. It has no operating system beneath
it in its target deployment, no allocator, no network stack, no filesystem, and
no secret material. That shape is the security story: there is very little to
steal and very little to corrupt.

## Assets

1. **Availability of the computation.** On an MCU a panic is a reset or a halt.
   The primary asset is *the core never panicking and never looping unbounded* on
   any input. Bounded, O(1) updates are a security property here, not just a
   performance one.
2. **Correctness / byte-parity.** The computed value must equal the `wickra-core`
   reference. A silent divergence that a downstream trading system trusts is the
   second asset.
3. **Memory safety across the C ABI.** The core is `#![forbid(unsafe_code)]`; the
   C ABI is the single `unsafe` surface. No caller input may cause an
   out-of-bounds access, a use of an invalid handle, or a panic unwinding into C.

## Trust boundaries

```
   untrusted numeric input (prices/candles from a feed)
                     │
        ┌────────────▼────────────┐   ← BOUNDARY 1: C ABI (unsafe surface)
        │   bindings/c            │      caller-supplied pointers & handles
        └────────────┬────────────┘
                     │  validated, panic-free
        ┌────────────▼────────────┐   ← BOUNDARY 2: no_std core (safe Rust)
        │   embed-core            │      forbid(unsafe_code), no alloc
        └─────────────────────────┘
```

- **Boundary 1 — the C ABI.** Callers pass raw pointers and stack-allocated
  handle storage. The FFI layer must reject null / misaligned pointers, treat
  every handle defensively, and let no panic cross into C. This is the only place
  `unsafe` lives and the main object of review.
- **Boundary 2 — the core.** Everything inside is safe Rust with no allocator.
  Its obligation is total: no panic, no unbounded loop, no NaN-propagation
  surprise on any finite or non-finite `f64` input.

## In scope

- A memory-safety or panic-across-FFI flaw in the C ABI or its handle protocol.
- Any input that makes the core panic, hang, or diverge from `wickra-core`.
- A dependency (`heapless`, `libm`) advisory that affects the shipped code paths.

## Out of scope

- Incorrect-but-deterministic indicator mathematics that matches `wickra-core`:
  a shared functional question, fixed in both, not a vulnerability here.
- Physical-access attacks on the target hardware (fault injection, glitching,
  probing the bus) — outside what a software library can defend.
- Misuse by the integrating firmware (feeding garbage prices, ignoring warmup):
  the library returns `None`/`Result`, but cannot police the caller's trading
  logic.
- Supply-chain compromise of the Rust toolchain or crates.io itself — mitigated
  by SHA-pinned CI actions, `cargo-deny`, and hash-checked dependencies, but not
  uniquely a `wickra-embed` concern.

## Mitigations in place

- `#![forbid(unsafe_code)]` on the core; `unsafe` confined to the C ABI and
  reviewed as the trust boundary.
- No allocator, no `panic!`/`unwrap`/`expect` in the core path — a panic on an
  MCU is a denial of service, so the code returns `Option`/`Result` instead.
- Byte-parity tests against `wickra-core` and property tests guard correctness;
  fuzzing exercises the update path on arbitrary input.
- `cargo-deny` + `osv-scanner` (VEX in `osv-scanner.toml`) gate advisories and
  licenses; CI actions are SHA-pinned; CodeQL scans the tree.
