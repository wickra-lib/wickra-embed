# The no-alloc C ABI handle contract

`wickra-embed` exposes its verified indicator subset to C and C++ firmware through
a C ABI that — unlike every other Wickra C ABI — **never calls `malloc`**. The
caller owns the storage for every indicator; the library only ever writes into
it. This document is the canonical description of that contract. The generated
header is [`bindings/c/include/wickra_embed.h`](../bindings/c/include/wickra_embed.h);
the per-binding quickstart lives in [`bindings/c/README.md`](../bindings/c/README.md).

## Why a caller-allocated handle

On a microcontroller there is often no allocator at all, and where there is one,
calling it on the hot path risks fragmentation and unbounded latency — the two
things `wickra-embed` exists to avoid. So the ABI never allocates. Instead, each
indicator is an **opaque, caller-allocated handle**: the caller places a buffer
(on the stack, in a static, or wherever it likes), and the library initialises
the indicator *into* that buffer and thereafter only reads and writes its bytes.
There is no `malloc` and no `free` anywhere in the surface — `reset` re-inits in
place, and dropping the handle is just letting the caller's buffer go out of
scope.

## Size and alignment are runtime accessors, not `#define`s

A handle's size and alignment are **target-dependent**: a `usize` is 4 bytes on a
Cortex-M and 8 on a 64-bit host, and a windowed indicator embeds its whole ring
buffer inline. A compile-time `#define` baked into the header would therefore be
wrong for some target the header is shared across. Instead the ABI exposes them as
runtime accessors:

```c
uintptr_t wickra_sma_size(void);   /* bytes of caller storage the handle needs   */
uintptr_t wickra_sma_align(void);  /* alignment (bytes) the handle needs          */
```

The contract for placing a handle is: provide a buffer of **at least
`wickra_sma_size()` bytes**, aligned to **at least `wickra_sma_align()` bytes**,
and `init` into it. On a real MCU you would size a static buffer from the accessor
for your target; in a portable example you over-provision and assert the runtime
values fit:

```c
#include <assert.h>
#include <stdalign.h>
#include "wickra_embed.h"

#define HANDLE_CAP   512  /* comfortably above any real handle on 32/64-bit */
#define HANDLE_ALIGN 16   /* covers the alignment of any scalar the handle holds */

assert(wickra_sma_size()  <= HANDLE_CAP);
assert(wickra_sma_align() <= HANDLE_ALIGN);

alignas(HANDLE_ALIGN) unsigned char storage[HANDLE_CAP];
WickraSma *sma = (WickraSma *) storage;
wickra_sma_init(sma);
```

A fixed `HANDLE_ALIGN` (rather than `alignof(max_align_t)`) keeps the example
portable: MSVC's C library does not define `max_align_t`, and `16` covers the
alignment of any scalar the handle can hold on 32- and 64-bit targets. The
runtime `assert` against `wickra_sma_align()` is what actually guarantees
correctness.

## The per-indicator surface

Every indicator (`sma`, `ema`, `rsi`, `atr`, `roc`) exposes the same seven-symbol
shape, prefixed with its name. Taking `sma`:

```c
uintptr_t   wickra_sma_size(void);
uintptr_t   wickra_sma_align(void);
int         wickra_sma_init(WickraSma *handle);
int         wickra_sma_update(WickraSma *handle, double input, double *out);
void        wickra_sma_reset(WickraSma *handle);
uintptr_t   wickra_sma_warmup(const WickraSma *handle);
int         wickra_sma_is_ready(const WickraSma *handle);
```

Plus one module-level symbol, valid in `no_std`:

```c
const char *wickra_embed_version(void);  /* static, NUL-terminated; never freed */
```

Two indicators deviate from the scalar `update(handle, price, out)` shape:

- **`ema`** takes a runtime period at init: `wickra_ema_init(handle, period)`.
  Its handle size is independent of the period (an EMA keeps no window), so one
  buffer size serves every period. A zero period returns `WICKRA_EMBED_ERR_PERIOD`.
- **`atr`** consumes an OHLC bar, not a single price:

  ```c
  int wickra_atr_update(WickraAtr *handle,
                        double open, double high, double low, double close,
                        double *out);
  ```

  All four prices must be finite (else `WICKRA_EMBED_ERR_NONFINITE`). Volume and
  timestamp do not affect ATR and are not part of the signature.

The exported windows match the golden reference set: **SMA(20)**, **EMA(period)**,
**RSI(14)**, **ATR(14)**, **ROC(10)**. `sma`, `rsi`, `atr`, `roc` fix the window
at compile time; only `ema` takes it at init.

## Return codes

`init` returns `WICKRA_EMBED_OK` (0) on success. `update` returns
`WICKRA_EMBED_READY` (1) when it wrote a value into `*out`, or
`WICKRA_EMBED_WARMUP` (0) while still warming up (`*out` untouched). Negative
values are errors:

| Return                        | Value | Meaning                                                  |
|-------------------------------|-------|----------------------------------------------------------|
| `WICKRA_EMBED_READY`          | `1`   | `update` wrote a value into `*out`.                      |
| `WICKRA_EMBED_OK` / `_WARMUP` | `0`   | `init` succeeded / `update` is still warming up.         |
| `WICKRA_EMBED_ERR_NULL`       | `-1`  | A required pointer (`handle` or `out`) is null.          |
| `WICKRA_EMBED_ERR_NONFINITE`  | `-2`  | An input price/OHLC value is NaN or infinite.            |
| `WICKRA_EMBED_ERR_PERIOD`     | `-3`  | `wickra_ema_init` was given a zero period.               |

Because `WICKRA_EMBED_OK` and `WICKRA_EMBED_WARMUP` are both `0`, test `init` for
`== WICKRA_EMBED_OK` and `update` for `== WICKRA_EMBED_READY`; treat any negative
return as an error.

## Null-safety and the warmup/ready pair

`reset`, `warmup`, `is_ready`, and the `size`/`align` accessors are **null-safe**:
`reset` on null is a no-op, and `warmup`/`is_ready` return `0`. This lets firmware
call them without a preceding null check. `wickra_sma_warmup()` returns the number
of inputs required before the first value; `wickra_sma_is_ready()` returns `1` if
the *next* update will yield a value. Note the SMA warmup convention: `Sma<N>`
emits its first value on the **N-th** update, so over `K` inputs there are
`K − N + 1` warm bars (41 for `N = 20` over 60 bars) — the C example asserts
exactly this.

## Byte-parity holds through the ABI

The C ABI is a thin `extern "C"` shim over `embed-core`; it performs no arithmetic
of its own. Every value it returns is therefore the **byte-for-byte identical**
value the std [`wickra-core`](https://github.com/wickra-lib/wickra) produces on a
server — the same bits on a Cortex-M0, a Cortex-M4F, and an x86-64 host. See
[PARITY.md](PARITY.md).

## Building: staticlib for firmware, cdylib for host

```bash
# Host build (staticlib + cdylib); links a panic handler via the default `std` feature:
cargo build -p wickra-embed-c --release

# Firmware build: link the staticlib; the firmware provides the panic handler:
cargo build -p wickra-embed-c --no-default-features --release --target thumbv7em-none-eabihf
```

Bare-metal targets build the `staticlib` (a `cdylib` needs a dynamic loader the
target does not have), so the release pipeline ships `libwickra_embed.a` per
triple plus the header. The header itself is generated by
[cbindgen](https://github.com/mozilla/cbindgen) and committed; CI diffs the
committed header against a fresh generation so the C surface can never drift from
the Rust surface silently.

## See also

- [NO_STD.md](NO_STD.md) — the `no_std` / no-alloc design the ABI sits on.
- [PARITY.md](PARITY.md) — how byte-parity with `wickra-core` is guaranteed.
- [LATENCY.md](LATENCY.md) — the bounded per-update latency the ABI inherits.
- [INDICATORS.md](INDICATORS.md) — the exported subset and how it grows.
