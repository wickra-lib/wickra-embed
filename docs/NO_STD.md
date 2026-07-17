# no_std and no-alloc design

`embed-core` is `#![no_std]` and allocation-free. This document explains what
that means concretely, how the crate is structured to hold the guarantee, and
which targets it is built and tested on.

## `no_std`, and one step further: no-alloc

Many `no_std` crates still use the `alloc` crate (`Box`, `Vec`, `String`) and so
require a global allocator. `embed-core` goes one step further: it uses **neither
`std` nor `alloc`**. There is no `Box`, no `Vec`, no `String`, no `HashMap`,
nowhere. Every indicator holds its entire state inline:

- Windowed indicators (`Sma`, `Rsi`, `Atr`, `Roc`) store their history in a
  const-generic fixed array — the window length is the type parameter `N`, so a
  `Sma<20>` is exactly 20 `f64` of buffer plus a few scalars, all on the stack or
  inside the caller's own storage.
- `Ema` needs no window at all: a running warmup sum and a count suffice, so it
  is allocation-free even with a runtime period.

The practical consequence: `embed-core` runs on a Cortex-M0 with a few KB of RAM,
with no allocator, no OS, and no surprises about heap fragmentation or
out-of-memory in the hot path.

## The feature switch

```toml
[features]
default = ["std"]
std = []
```

- **Host builds** (`cargo build`, `cargo test`) use the default `std` feature.
  This does *not* make the core use the heap — it only lets the standard-library
  float methods back the `math` module and lets the test/doc harness run.
- **Bare-metal builds** use `--no-default-features`. The crate is then strictly
  `#![no_std]`, and the few f64 operations `core` lacks route through `libm`.

There is deliberately **no `alloc` feature**. Adding one would be the first step
toward a heap; the whole point is not to have one.

## The `math` switch point

`core` provides `+ - * /` on `f64` but not `sqrt`, `fabs`, `fmax`, or the fused
multiply-add. `embed-core` funnels the ones it needs through a single module,
`math`, with two implementations selected by the `std` feature:

| Operation | `std` path | `no_std` path |
|-----------|------------|---------------|
| `abs`     | `f64::abs` | `libm::fabs`  |
| `max`     | `f64::max` | `libm::fmax`  |
| `mul_add` | `f64::mul_add` | `libm::fma` |

The two paths are IEEE-754 identical for these operations: `fma` is the uniquely
correctly-rounded fused multiply-add, `fabs` clears the sign bit, and `fmax`
matches `f64::max` on finite inputs. The `math` unit tests and the parity tests
assert the equality directly, so the host and bare-metal builds compute the same
bits. `Sma`, `Ema` and `Roc` need only `+ - * /` and touch `math` not at all.

## No panics in the update path

On a microcontroller a panic is a reset or a halt — a denial of service. The core
therefore never panics on the hot path:

- no `unwrap`, `expect`, `unreachable!`, or indexing that can go out of bounds;
- warmup and undefined states are expressed as `None`, never a panic;
- `#![forbid(unsafe_code)]` on the whole crate.

A `no_std` *binary* (the QEMU/Cortex-M example) still needs a `#[panic_handler]`
and `panic = "abort"`, but the library itself never reaches one.

## Targets

The core is built on every change against, and byte-identical across:

| Target | Meaning |
|--------|---------|
| `thumbv7em-none-eabihf` | Cortex-M4F / M7, hardware FPU (f32; f64 in software) |
| `thumbv6m-none-eabi`    | Cortex-M0 / M0+, no FPU, all soft-float |
| `x86_64-*` (host)       | tests, doctests and benches (std) |

The two bare-metal targets are pinned in `rust-toolchain.toml`, so `cargo build
--no-default-features --target …` and CI pick them up automatically. Because f64
`+ - * /` is IEEE-754 deterministic and the core uses the exact same operation
order everywhere, the value on a soft-float Cortex-M0 is bit-for-bit the value on
the host — see [PARITY.md](PARITY.md).
