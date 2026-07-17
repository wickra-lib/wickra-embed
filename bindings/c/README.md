# Wickra Embed — C ABI

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/wickra-lib/wickra-embed#license)
[![no_std](https://img.shields.io/badge/no__std-yes-success.svg)](https://github.com/wickra-lib/wickra-embed/blob/main/docs/NO_STD.md)
[![no-alloc](https://img.shields.io/badge/alloc-none-success.svg)](https://github.com/wickra-lib/wickra-embed/blob/main/docs/NO_STD.md)
[![byte-parity: wickra-core](https://img.shields.io/badge/byte--parity-wickra--core-success.svg)](https://github.com/wickra-lib/wickra-embed/blob/main/docs/PARITY.md)

The **no-alloc** C ABI is the hub bare-metal C and C++ firmware links against. It
exposes the verified [`embed-core`](https://github.com/wickra-lib/wickra-embed)
indicator subset as a `staticlib` (to link into firmware) and a `cdylib` (for
host use), and — unlike the other Wickra C ABIs — it **never calls `malloc`**.
The caller owns the storage for every indicator; the library only writes into it.

## Surface

Each indicator (`sma`, `ema`, `rsi`, `atr`, `roc`) exposes the same shape. Taking
`sma` as the example:

```c
#include "wickra_embed.h"

uintptr_t wickra_sma_size(void);                 /* bytes of caller storage a handle needs */
uintptr_t wickra_sma_align(void);                /* alignment (bytes) a handle needs        */
int       wickra_sma_init(WickraSma *handle);    /* init a fresh SMA(20) into caller storage */
int       wickra_sma_update(WickraSma *handle, double input, double *out);
void      wickra_sma_reset(WickraSma *handle);
uintptr_t wickra_sma_warmup(const WickraSma *handle);
int       wickra_sma_is_ready(const WickraSma *handle);

const char *wickra_embed_version(void);          /* static, NUL-terminated; do not free */
```

The exported windows match the golden reference set: **SMA(20)**, **EMA(period)**,
**RSI(14)**, **ATR(14)**, **ROC(10)**. `ema` takes a runtime period at init
(`wickra_ema_init(handle, period)`); the others fix the window at compile time.
`atr` consumes an OHLC bar rather than a single price:

```c
int wickra_atr_update(WickraAtr *handle, double open, double high, double low, double close, double *out);
```

## The handle contract (init-into-caller-storage)

Handles are **opaque and caller-allocated**. Their size and alignment depend on
the target (a `usize` is 4 bytes on a Cortex-M, 8 on a 64-bit host), so they are
exposed as **runtime accessors** rather than compile-time `#define`s. Query the
size and alignment, place a buffer, and `init` into it — there is no `malloc` and
no `free`:

```c
#include <stdio.h>
#include "wickra_embed.h"

int main(void) {
    /* A buffer at least wickra_sma_size() bytes, wickra_sma_align()-aligned.
       On a real MCU you would size it from wickra_sma_size() for your target. */
    _Alignas(8) unsigned char buf[64];
    WickraSma *h = (WickraSma *)buf;

    if (wickra_sma_init(h) != WICKRA_EMBED_OK) return 1;

    double out;
    for (double price = 1.0; price <= 20.0; price += 1.0) {
        int rc = wickra_sma_update(h, price, &out);
        if (rc == WICKRA_EMBED_READY) {
            printf("SMA(20) = %.4f\n", out);   /* prints 10.5 on the 20th bar */
        }
    }
    return 0;
}
```

## Return codes

`init` returns `WICKRA_EMBED_OK` (0) on success. `update` returns
`WICKRA_EMBED_READY` (1) when it wrote a value into `out`, or
`WICKRA_EMBED_WARMUP` (0) while still warming up (`out` untouched). Negative
values are errors:

| Return                        | Value | Meaning                                                        |
|-------------------------------|-------|----------------------------------------------------------------|
| `WICKRA_EMBED_READY`          | `1`   | `update` wrote a value into `*out`.                            |
| `WICKRA_EMBED_OK` / `_WARMUP` | `0`   | `init` succeeded / `update` is still warming up.               |
| `WICKRA_EMBED_ERR_NULL`       | `-1`  | A required pointer (`handle` or `out`) is null.                |
| `WICKRA_EMBED_ERR_NONFINITE`  | `-2`  | An input price/OHLC value is NaN or infinite.                  |
| `WICKRA_EMBED_ERR_PERIOD`     | `-3`  | `wickra_ema_init` was given a zero period.                     |

`reset`, `warmup`, `is_ready` and the `size`/`align` accessors are null-safe
(`reset` on null is a no-op; `warmup`/`is_ready` return 0).

## Byte-parity

Every value this ABI produces is **byte-for-byte identical** to the value the std
[`wickra-core`](https://github.com/wickra-lib/wickra) produces on a server — the
same bits on a Cortex-M0, a Cortex-M4F and an x86-64 host. See
[docs/PARITY.md](https://github.com/wickra-lib/wickra-embed/blob/main/docs/PARITY.md).

## Building

```bash
# Host build (staticlib + cdylib), links a panic handler via the default `std` feature:
cargo build -p wickra-embed-c --release

# Firmware build: link the staticlib and let the firmware provide the panic handler:
cargo build -p wickra-embed-c --no-default-features --release --target thumbv7em-none-eabihf
```

The header `include/wickra_embed.h` is generated by
[cbindgen](https://github.com/mozilla/cbindgen) and committed; CI checks it stays
in sync with the Rust surface.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-embed/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-embed/blob/main/LICENSE-APACHE),
at your option.
