# Architecture

`wickra-embed` is a small `#![no_std]`, allocation-free core with a thin C ABI on
top. Its whole reason to exist is a single guarantee: **the value it computes on
bare metal is byte-for-byte identical to the value the std `wickra-core` computes
on a server.**

## The layers

```
CONSUMERS   firmware (C/C++) via the no-alloc C ABI   ·   Rust via embed-core directly
      ▲ Option<f64> per update                                   ▲
CORE  crates/embed-core (#![no_std], no alloc):  Indicator contract → const-generic
                             ring buffer → Sma/Ema/Rsi/Atr/Roc, O(1) bounded-latency update
      ▼ no-alloc, stack-handle C ABI (bindings/c)
TARGETS  thumbv7em-none-eabihf (Cortex-M4F/M7)  ·  thumbv6m-none-eabi (Cortex-M0/M0+)  ·  x86-64 host
PARITY ORACLE  wickra-core (std) — dev-dependency only, never in the core path
```

## The core is allocation-free

`embed-core` is `#![no_std]` and `#![forbid(unsafe_code)]`. It never touches an
allocator: buffers are `const`-generic fixed-capacity ring buffers, so an
indicator's entire state lives inline in its struct. There is no `Box`, no `Vec`,
no `String`, no `HashMap`. `wickra-core` — which does use `alloc` — appears only
under `[dev-dependencies]`, as the reference the parity tests compare against; it
is never linked into the bare-metal build.

## The `Indicator` contract

Every indicator implements the same trait `wickra-core` uses — `update`, `reset`,
`warmup_period`, `is_ready`, `name` — with an associated `Input: Copy` and an
`Option<f64>` output that is `None` during warmup. The only thing that differs
from `wickra-core` is the storage: a heap `Box<[f64]>` there, a const-generic
buffer here. The arithmetic is deliberately identical.

## The no-alloc C ABI

`bindings/c` exposes the core to firmware written in C. Handles are
**stack-allocated by the caller** (init-into-storage), not heap-allocated, so the
FFI layer allocates nothing either. It is the one crate that re-allows `unsafe`;
it wraps the boundary so no panic and no invalid pointer crosses into C.

## Determinism (the moat)

The byte-parity holds because the floating-point work is done in a fixed order,
identical to `wickra-core`:

- The same rolling-sum add/subtract order and the same periodic reseed cadence
  against f64 drift.
- The final `sum / period` division, not an incrementally maintained mean.
- No RNG, no time, no `HashMap` iteration order, no threads — nothing whose
  result depends on the platform.
- The `std::f64` math path (host) and the `libm` path (bare metal) are held to
  the same values by the parity tests, so `libm` cannot silently diverge.

Every finite result is the exact `wickra-core` result; any divergence is a bug.

## Why no 10-language binding surface

The other Wickra products ship a JSON-over-C-ABI hub to ten languages. That hub
allocates (JSON strings, argument buffers) and assumes an OS. `wickra-embed`
deliberately does **not** ship it: the moat here is not breadth of languages but
zero allocation on hardware that has no allocator. C is the only foreign surface,
because C is the language firmware is written in. A host-side Python/Node binding
is a possible future convenience, not a v0.1 goal (see [ROADMAP.md](ROADMAP.md)).

## Integration with the rest of Wickra

`wickra-embed` depends on nothing at runtime beyond `libm` (for the transcendental
f64 functions on targets without them) and `heapless`. It places no orders, opens
no connections, and holds no secret material.
