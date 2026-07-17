//! The wickra-embed no-alloc C ABI — the hub bare-metal C/C++ firmware links against.
//!
//! The surface is deliberately tiny and **allocation-free**. Unlike the other
//! Wickra C ABIs, nothing here is JSON-shaped and nothing is heap-allocated: the
//! caller owns the storage for every indicator handle, and the library only ever
//! writes into that storage. This is the contract a microcontroller or an HFT
//! co-processor needs — fixed latency, no allocator, no fragmentation.
//!
//! # Handle contract (init-into-caller-storage)
//!
//! Each indicator is an opaque, fixed-size handle the caller places on its own
//! stack or in static storage. The size and alignment of a handle depend on the
//! target (a `usize` is 4 bytes on a Cortex-M, 8 on a 64-bit host), so they are
//! exposed as **runtime accessors** rather than compile-time `#define`s — query
//! [`wickra_sma_size`] / [`wickra_sma_align`] and friends, place a buffer of that
//! size and alignment, then `init` into it:
//!
//! ```c
//! #include "wickra_embed.h"
//! _Alignas(8) unsigned char buf[64];          /* >= wickra_sma_size(), wickra_sma_align()-aligned */
//! WickraSma *h = (WickraSma *)buf;
//! wickra_sma_init(h);
//! double out;
//! if (wickra_sma_update(h, price, &out) == WICKRA_EMBED_READY) { /* out is valid */ }
//! ```
//!
//! The library **never** calls `malloc`: `init` does a `ptr::write` of a fresh
//! state into the caller's block. There is no `free` because there is nothing to
//! free — the storage is the caller's.
//!
//! # Return codes
//!
//! - `init` returns [`WICKRA_EMBED_OK`] (0) on success, [`WICKRA_EMBED_ERR_NULL`]
//!   (-1) if the handle is null, and (for `ema`) [`WICKRA_EMBED_ERR_PERIOD`] (-3)
//!   if `period == 0`.
//! - `update` returns [`WICKRA_EMBED_READY`] (1) when it wrote a value into `out`,
//!   [`WICKRA_EMBED_WARMUP`] (0) while still warming up (`out` untouched),
//!   [`WICKRA_EMBED_ERR_NULL`] (-1) if `handle` or `out` is null, and
//!   [`WICKRA_EMBED_ERR_NONFINITE`] (-2) if an input is NaN or infinite (the core
//!   never admits a non-finite value into its state).
//! - `reset`, `warmup`, `is_ready` and the `size`/`align` accessors are null-safe.
//!
//! # No `catch_unwind`
//!
//! There is no panic boundary here: the core is written not to panic (no
//! `unwrap`, no out-of-bounds indexing, warmup expressed as `None`), and the
//! release profile is `panic = "abort"`. A `no_std` target has no unwinding
//! machinery to catch anyway.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unsafe_code)]

use core::ffi::{c_char, c_int};

use embed_core::{Atr, Candle, Ema, Indicator, Roc, Rsi, Sma};

// Fixed window sizes per exported type. These match the golden reference set
// (`sma20`, `ema20`, `rsi14`, `atr14`, `roc10`); additional windows would be
// additional exported symbols.
type SmaN = Sma<20>;
type RsiN = Rsi<14>;
type AtrN = Atr<14>;
type RocN = Roc<10>;

/// Success return from an `init` call.
pub const WICKRA_EMBED_OK: c_int = 0;
/// An `update` produced a value (written into `out`).
pub const WICKRA_EMBED_READY: c_int = 1;
/// An `update` is still warming up; `out` is untouched.
pub const WICKRA_EMBED_WARMUP: c_int = 0;
/// A required pointer argument was null.
pub const WICKRA_EMBED_ERR_NULL: c_int = -1;
/// An input price/OHLC value was NaN or infinite.
pub const WICKRA_EMBED_ERR_NONFINITE: c_int = -2;
/// An `ema` init was asked for a zero period.
pub const WICKRA_EMBED_ERR_PERIOD: c_int = -3;

/// The library version as a static, NUL-terminated C string. Valid in `no_std`.
#[no_mangle]
pub extern "C" fn wickra_embed_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}

// --- SMA (Simple Moving Average, window 20, input f64) ----------------------

/// An opaque, caller-allocated SMA(20) handle. Place a buffer of
/// [`wickra_sma_size`] bytes at [`wickra_sma_align`] alignment and `init` into it.
pub struct WickraSma(SmaN);

/// Bytes of caller storage a [`WickraSma`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_sma_size() -> usize {
    core::mem::size_of::<WickraSma>()
}

/// Alignment (bytes) a [`WickraSma`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_sma_align() -> usize {
    core::mem::align_of::<WickraSma>()
}

/// Initialise a fresh SMA(20) into caller storage. `handle` must point to
/// [`wickra_sma_size`] bytes, [`wickra_sma_align`]-aligned.
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_init(handle: *mut WickraSma) -> c_int {
    if handle.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    unsafe { core::ptr::write(handle, WickraSma(SmaN::new())) };
    WICKRA_EMBED_OK
}

/// Feed one price. Writes the average into `*out` and returns
/// [`WICKRA_EMBED_READY`] once warm, else [`WICKRA_EMBED_WARMUP`].
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_update(
    handle: *mut WickraSma,
    input: f64,
    out: *mut f64,
) -> c_int {
    if handle.is_null() || out.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    if !input.is_finite() {
        return WICKRA_EMBED_ERR_NONFINITE;
    }
    let sma = unsafe { &mut *handle };
    match sma.0.update(input) {
        Some(v) => {
            unsafe { *out = v };
            WICKRA_EMBED_READY
        }
        None => WICKRA_EMBED_WARMUP,
    }
}

/// Reset the handle to its freshly-initialised state. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_reset(handle: *mut WickraSma) {
    if !handle.is_null() {
        unsafe { (*handle).0.reset() };
    }
}

/// Number of inputs before the first value. Null-safe (returns 0).
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_warmup(handle: *const WickraSma) -> usize {
    if handle.is_null() {
        0
    } else {
        unsafe { (*handle).0.warmup_period() }
    }
}

/// 1 if the next update will yield a value, else 0. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_is_ready(handle: *const WickraSma) -> c_int {
    c_int::from(!handle.is_null() && unsafe { (*handle).0.is_ready() })
}

// --- EMA (Exponential Moving Average, runtime period, input f64) ------------

/// An opaque, caller-allocated EMA handle. Its size is independent of the period
/// (an EMA keeps no window), so one buffer size serves every period.
pub struct WickraEma(Ema);

/// Bytes of caller storage a [`WickraEma`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_ema_size() -> usize {
    core::mem::size_of::<WickraEma>()
}

/// Alignment (bytes) a [`WickraEma`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_ema_align() -> usize {
    core::mem::align_of::<WickraEma>()
}

/// Initialise a fresh EMA of the given `period` into caller storage. Returns
/// [`WICKRA_EMBED_ERR_PERIOD`] if `period == 0`.
#[no_mangle]
pub unsafe extern "C" fn wickra_ema_init(handle: *mut WickraEma, period: usize) -> c_int {
    if handle.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    if period == 0 {
        return WICKRA_EMBED_ERR_PERIOD;
    }
    unsafe { core::ptr::write(handle, WickraEma(Ema::new(period))) };
    WICKRA_EMBED_OK
}

/// Feed one price. See [`wickra_sma_update`] for the return contract.
#[no_mangle]
pub unsafe extern "C" fn wickra_ema_update(
    handle: *mut WickraEma,
    input: f64,
    out: *mut f64,
) -> c_int {
    if handle.is_null() || out.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    if !input.is_finite() {
        return WICKRA_EMBED_ERR_NONFINITE;
    }
    let ema = unsafe { &mut *handle };
    match ema.0.update(input) {
        Some(v) => {
            unsafe { *out = v };
            WICKRA_EMBED_READY
        }
        None => WICKRA_EMBED_WARMUP,
    }
}

/// Reset the handle. Null-safe. The period set at `init` is preserved.
#[no_mangle]
pub unsafe extern "C" fn wickra_ema_reset(handle: *mut WickraEma) {
    if !handle.is_null() {
        unsafe { (*handle).0.reset() };
    }
}

/// Number of inputs before the first value. Null-safe (returns 0).
#[no_mangle]
pub unsafe extern "C" fn wickra_ema_warmup(handle: *const WickraEma) -> usize {
    if handle.is_null() {
        0
    } else {
        unsafe { (*handle).0.warmup_period() }
    }
}

/// 1 if the next update will yield a value, else 0. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_ema_is_ready(handle: *const WickraEma) -> c_int {
    c_int::from(!handle.is_null() && unsafe { (*handle).0.is_ready() })
}

// --- RSI (Wilder, window 14, input f64) -------------------------------------

/// An opaque, caller-allocated RSI(14) handle.
pub struct WickraRsi(RsiN);

/// Bytes of caller storage a [`WickraRsi`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_rsi_size() -> usize {
    core::mem::size_of::<WickraRsi>()
}

/// Alignment (bytes) a [`WickraRsi`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_rsi_align() -> usize {
    core::mem::align_of::<WickraRsi>()
}

/// Initialise a fresh RSI(14) into caller storage.
#[no_mangle]
pub unsafe extern "C" fn wickra_rsi_init(handle: *mut WickraRsi) -> c_int {
    if handle.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    unsafe { core::ptr::write(handle, WickraRsi(RsiN::new())) };
    WICKRA_EMBED_OK
}

/// Feed one price. See [`wickra_sma_update`] for the return contract.
#[no_mangle]
pub unsafe extern "C" fn wickra_rsi_update(
    handle: *mut WickraRsi,
    input: f64,
    out: *mut f64,
) -> c_int {
    if handle.is_null() || out.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    if !input.is_finite() {
        return WICKRA_EMBED_ERR_NONFINITE;
    }
    let rsi = unsafe { &mut *handle };
    match rsi.0.update(input) {
        Some(v) => {
            unsafe { *out = v };
            WICKRA_EMBED_READY
        }
        None => WICKRA_EMBED_WARMUP,
    }
}

/// Reset the handle. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_rsi_reset(handle: *mut WickraRsi) {
    if !handle.is_null() {
        unsafe { (*handle).0.reset() };
    }
}

/// Number of inputs before the first value. Null-safe (returns 0).
#[no_mangle]
pub unsafe extern "C" fn wickra_rsi_warmup(handle: *const WickraRsi) -> usize {
    if handle.is_null() {
        0
    } else {
        unsafe { (*handle).0.warmup_period() }
    }
}

/// 1 if the next update will yield a value, else 0. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_rsi_is_ready(handle: *const WickraRsi) -> c_int {
    c_int::from(!handle.is_null() && unsafe { (*handle).0.is_ready() })
}

// --- ATR (Wilder, window 14, input OHLC candle) -----------------------------

/// An opaque, caller-allocated ATR(14) handle.
pub struct WickraAtr(AtrN);

/// Bytes of caller storage a [`WickraAtr`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_atr_size() -> usize {
    core::mem::size_of::<WickraAtr>()
}

/// Alignment (bytes) a [`WickraAtr`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_atr_align() -> usize {
    core::mem::align_of::<WickraAtr>()
}

/// Initialise a fresh ATR(14) into caller storage.
#[no_mangle]
pub unsafe extern "C" fn wickra_atr_init(handle: *mut WickraAtr) -> c_int {
    if handle.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    unsafe { core::ptr::write(handle, WickraAtr(AtrN::new())) };
    WICKRA_EMBED_OK
}

/// Feed one OHLC bar. The four prices must all be finite (else
/// [`WICKRA_EMBED_ERR_NONFINITE`]). Volume and timestamp do not affect ATR and
/// are not part of the signature.
#[no_mangle]
pub unsafe extern "C" fn wickra_atr_update(
    handle: *mut WickraAtr,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    out: *mut f64,
) -> c_int {
    if handle.is_null() || out.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    if !(open.is_finite() && high.is_finite() && low.is_finite() && close.is_finite()) {
        return WICKRA_EMBED_ERR_NONFINITE;
    }
    let atr = unsafe { &mut *handle };
    let candle = Candle::new(open, high, low, close, 0.0, 0);
    match atr.0.update(candle) {
        Some(v) => {
            unsafe { *out = v };
            WICKRA_EMBED_READY
        }
        None => WICKRA_EMBED_WARMUP,
    }
}

/// Reset the handle. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_atr_reset(handle: *mut WickraAtr) {
    if !handle.is_null() {
        unsafe { (*handle).0.reset() };
    }
}

/// Number of candles before the first value. Null-safe (returns 0).
#[no_mangle]
pub unsafe extern "C" fn wickra_atr_warmup(handle: *const WickraAtr) -> usize {
    if handle.is_null() {
        0
    } else {
        unsafe { (*handle).0.warmup_period() }
    }
}

/// 1 if the next update will yield a value, else 0. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_atr_is_ready(handle: *const WickraAtr) -> c_int {
    c_int::from(!handle.is_null() && unsafe { (*handle).0.is_ready() })
}

// --- ROC (Rate of Change %, window 10, input f64) ---------------------------

/// An opaque, caller-allocated ROC(10) handle.
pub struct WickraRoc(RocN);

/// Bytes of caller storage a [`WickraRoc`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_roc_size() -> usize {
    core::mem::size_of::<WickraRoc>()
}

/// Alignment (bytes) a [`WickraRoc`] handle needs on this target.
#[no_mangle]
pub extern "C" fn wickra_roc_align() -> usize {
    core::mem::align_of::<WickraRoc>()
}

/// Initialise a fresh ROC(10) into caller storage.
#[no_mangle]
pub unsafe extern "C" fn wickra_roc_init(handle: *mut WickraRoc) -> c_int {
    if handle.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    unsafe { core::ptr::write(handle, WickraRoc(RocN::new())) };
    WICKRA_EMBED_OK
}

/// Feed one price. See [`wickra_sma_update`] for the return contract.
#[no_mangle]
pub unsafe extern "C" fn wickra_roc_update(
    handle: *mut WickraRoc,
    input: f64,
    out: *mut f64,
) -> c_int {
    if handle.is_null() || out.is_null() {
        return WICKRA_EMBED_ERR_NULL;
    }
    if !input.is_finite() {
        return WICKRA_EMBED_ERR_NONFINITE;
    }
    let roc = unsafe { &mut *handle };
    match roc.0.update(input) {
        Some(v) => {
            unsafe { *out = v };
            WICKRA_EMBED_READY
        }
        None => WICKRA_EMBED_WARMUP,
    }
}

/// Reset the handle. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_roc_reset(handle: *mut WickraRoc) {
    if !handle.is_null() {
        unsafe { (*handle).0.reset() };
    }
}

/// Number of inputs before the first value. Null-safe (returns 0).
#[no_mangle]
pub unsafe extern "C" fn wickra_roc_warmup(handle: *const WickraRoc) -> usize {
    if handle.is_null() {
        0
    } else {
        unsafe { (*handle).0.warmup_period() }
    }
}

/// 1 if the next update will yield a value, else 0. Null-safe.
#[no_mangle]
pub unsafe extern "C" fn wickra_roc_is_ready(handle: *const WickraRoc) -> c_int {
    c_int::from(!handle.is_null() && unsafe { (*handle).0.is_ready() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::CStr;
    use core::mem::MaybeUninit;
    use core::ptr;

    // Initialise a handle into a stack `MaybeUninit`, exactly as a C caller would
    // place it in `_Alignas(..) unsigned char buf[..]`.
    fn sma() -> MaybeUninit<WickraSma> {
        let mut slot = MaybeUninit::<WickraSma>::uninit();
        assert_eq!(
            unsafe { wickra_sma_init(slot.as_mut_ptr()) },
            WICKRA_EMBED_OK
        );
        slot
    }

    #[test]
    fn version_is_the_package_version() {
        let ptr = wickra_embed_version();
        let s = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap();
        assert_eq!(s, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn size_and_align_are_the_wrapped_type() {
        assert_eq!(wickra_sma_size(), core::mem::size_of::<WickraSma>());
        assert_eq!(wickra_sma_align(), core::mem::align_of::<WickraSma>());
        assert!(wickra_ema_size() >= core::mem::size_of::<f64>());
        assert!(wickra_atr_align() >= 1);
    }

    #[test]
    fn init_rejects_null() {
        unsafe {
            assert_eq!(wickra_sma_init(ptr::null_mut()), WICKRA_EMBED_ERR_NULL);
            assert_eq!(wickra_ema_init(ptr::null_mut(), 20), WICKRA_EMBED_ERR_NULL);
            assert_eq!(wickra_atr_init(ptr::null_mut()), WICKRA_EMBED_ERR_NULL);
        }
    }

    #[test]
    fn ema_rejects_zero_period() {
        let mut slot = MaybeUninit::<WickraEma>::uninit();
        assert_eq!(
            unsafe { wickra_ema_init(slot.as_mut_ptr(), 0) },
            WICKRA_EMBED_ERR_PERIOD
        );
    }

    #[test]
    fn update_rejects_null_handle_and_out() {
        let mut slot = sma();
        let mut out = 0.0_f64;
        unsafe {
            assert_eq!(
                wickra_sma_update(ptr::null_mut(), 1.0, &mut out),
                WICKRA_EMBED_ERR_NULL
            );
            assert_eq!(
                wickra_sma_update(slot.as_mut_ptr(), 1.0, ptr::null_mut()),
                WICKRA_EMBED_ERR_NULL
            );
        }
    }

    #[test]
    fn update_rejects_non_finite() {
        let mut slot = sma();
        let mut out = 0.0_f64;
        unsafe {
            assert_eq!(
                wickra_sma_update(slot.as_mut_ptr(), f64::NAN, &mut out),
                WICKRA_EMBED_ERR_NONFINITE
            );
            assert_eq!(
                wickra_sma_update(slot.as_mut_ptr(), f64::INFINITY, &mut out),
                WICKRA_EMBED_ERR_NONFINITE
            );
        }
    }

    #[test]
    fn atr_rejects_non_finite_ohlc() {
        let mut slot = MaybeUninit::<WickraAtr>::uninit();
        let mut out = 0.0_f64;
        unsafe {
            assert_eq!(wickra_atr_init(slot.as_mut_ptr()), WICKRA_EMBED_OK);
            assert_eq!(
                wickra_atr_update(slot.as_mut_ptr(), 1.0, f64::NAN, 1.0, 1.0, &mut out),
                WICKRA_EMBED_ERR_NONFINITE
            );
        }
    }

    #[test]
    fn sma_warms_up_then_produces_a_value() {
        let mut slot = sma();
        let h = slot.as_mut_ptr();
        let mut out = 0.0_f64;
        unsafe {
            assert_eq!(wickra_sma_warmup(h), 20);
            // First 19 updates warm up; the 20th produces the average.
            for i in 1..=19 {
                assert_eq!(
                    wickra_sma_update(h, f64::from(i), &mut out),
                    WICKRA_EMBED_WARMUP
                );
                assert_eq!(wickra_sma_is_ready(h), 0);
            }
            assert_eq!(wickra_sma_update(h, 20.0, &mut out), WICKRA_EMBED_READY);
            assert_eq!(wickra_sma_is_ready(h), 1);
        }
        assert!((out - 10.5).abs() < 1e-9); // mean of 1..=20
    }

    #[test]
    fn reset_returns_to_warmup() {
        let mut slot = sma();
        let h = slot.as_mut_ptr();
        let mut out = 0.0_f64;
        unsafe {
            for i in 1..=20 {
                wickra_sma_update(h, f64::from(i), &mut out);
            }
            assert_eq!(wickra_sma_is_ready(h), 1);
            wickra_sma_reset(h);
            assert_eq!(wickra_sma_is_ready(h), 0);
            assert_eq!(wickra_sma_update(h, 1.0, &mut out), WICKRA_EMBED_WARMUP);
        }
    }

    #[test]
    fn null_safe_accessors() {
        unsafe {
            assert_eq!(wickra_sma_warmup(ptr::null()), 0);
            assert_eq!(wickra_sma_is_ready(ptr::null()), 0);
            wickra_sma_reset(ptr::null_mut()); // no-op, must not crash
        }
    }

    #[test]
    fn ema_and_roc_and_rsi_produce_values() {
        let mut out = 0.0_f64;
        unsafe {
            // EMA(3): ready after 3 inputs.
            let mut e = MaybeUninit::<WickraEma>::uninit();
            assert_eq!(wickra_ema_init(e.as_mut_ptr(), 3), WICKRA_EMBED_OK);
            let mut ready = WICKRA_EMBED_WARMUP;
            for i in 1..=5 {
                ready = wickra_ema_update(e.as_mut_ptr(), f64::from(i), &mut out);
            }
            assert_eq!(ready, WICKRA_EMBED_READY);

            // ROC(10): ready after 11 inputs.
            let mut r = MaybeUninit::<WickraRoc>::uninit();
            assert_eq!(wickra_roc_init(r.as_mut_ptr()), WICKRA_EMBED_OK);
            assert_eq!(wickra_roc_warmup(r.as_mut_ptr()), 11);

            // RSI(14): ready after 15 inputs.
            let mut s = MaybeUninit::<WickraRsi>::uninit();
            assert_eq!(wickra_rsi_init(s.as_mut_ptr()), WICKRA_EMBED_OK);
            assert_eq!(wickra_rsi_warmup(s.as_mut_ptr()), 15);
        }
    }
}
