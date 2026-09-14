// Optional C++ convenience layer over the wickra-embed C ABI (`wickra_embed.h`).
//
// The C ABI is no-alloc by contract: the caller owns the handle's storage and
// asks the library at run time how large and how aligned it has to be. That is
// the right shape for firmware and easy to get subtly wrong in C++ -- an
// under-sized buffer, a forgotten init, a status code read as a value. This
// header keeps every one of those properties (no heap, no exceptions, no RTTI,
// nothing the bare-metal targets lack) and adds the one thing C++ can give:
// a type per indicator whose storage is part of the object.
//
//     #include "wickra_embed.hpp"
//
//     wickra_embed::Sma sma;              // the handle lives inside `sma`
//     for (double p : prices) {
//         auto r = sma.update(p);
//         if (r.ready) act_on(r.value); // byte-identical to wickra-core
//     }
//
// `CAPACITY` is a compile-time bound on the handle size; the constructor checks
// the run-time size against it and refuses (`ok()` is false, `update` reports
// `WICKRA_EMBED_ERR_NULL`) rather than write past the buffer. 512 bytes with a
// 16-byte alignment covers every indicator in the v0.1 subset on 32- and 64-bit
// targets; `wickra_sma_size()` and friends are the source of truth.
//
// Header-only, and adds no runtime cost beyond the C calls themselves.
#ifndef WICKRA_EMBED_HPP
#define WICKRA_EMBED_HPP

#include "wickra_embed.h"

#include <cstddef>
#include <cstdint>

namespace wickra_embed {

/// One update's outcome: `ready` is true once the indicator is warm and
/// `value` then holds the output; `status` is the raw C ABI code, negative on
/// error (`WICKRA_EMBED_ERR_NONFINITE` for a NaN or infinite input).
struct Update {
    double value;
    int status;
    bool ready;
};

/// Storage-owning wrapper over one C ABI handle type. `Traits` binds the five
/// C entry points of an indicator; the aliases below are the public surface.
template <typename Traits, std::size_t CAPACITY = 512, std::size_t ALIGN = 16>
class Handle {
public:
    Handle() : status_(WICKRA_EMBED_ERR_NULL) {
        if (Traits::size() <= CAPACITY && Traits::align() <= ALIGN) {
            status_ = Traits::init(raw());
        }
    }

    // The handle is a value: copying it copies the indicator's state, which is
    // exactly the C ABI's semantics for the bytes behind the pointer.
    Handle(const Handle&) = default;
    Handle& operator=(const Handle&) = default;

    /// False when the buffer could not hold the handle or init failed; every
    /// other call then reports `WICKRA_EMBED_ERR_NULL`.
    bool ok() const { return status_ == WICKRA_EMBED_OK; }

    /// Return to the freshly initialised state.
    void reset() {
        if (ok()) Traits::reset(raw());
    }

    /// Inputs still needed before the first value; 0 once warm.
    std::size_t warmup() const {
        return ok() ? static_cast<std::size_t>(Traits::warmup(raw())) : 0;
    }

    bool is_ready() const { return ok() && Traits::is_ready(raw()) != 0; }

    /// Raw access for code that wants the C API on this storage.
    typename Traits::Raw* raw() { return reinterpret_cast<typename Traits::Raw*>(storage_); }
    const typename Traits::Raw* raw() const { return reinterpret_cast<const typename Traits::Raw*>(storage_); }

protected:
    Update finish(int status, double out) const {
        Update u;
        u.value = out;
        u.status = status;
        u.ready = status == WICKRA_EMBED_READY;
        return u;
    }

    int status_;
    alignas(ALIGN) unsigned char storage_[CAPACITY];
};

#define WICKRA_EMBED_TRAITS(NAME, CNAME, PREFIX)                                          \
    struct NAME##Traits {                                                                 \
        using Raw = CNAME;                                                                \
        static std::size_t size() { return static_cast<std::size_t>(PREFIX##_size()); }   \
        static std::size_t align() { return static_cast<std::size_t>(PREFIX##_align()); } \
        static int init(Raw* h) { return PREFIX##_init(h); }                              \
        static void reset(Raw* h) { PREFIX##_reset(h); }                                  \
        static uintptr_t warmup(const Raw* h) { return PREFIX##_warmup(h); }              \
        static int is_ready(const Raw* h) { return PREFIX##_is_ready(h); }                \
    };

WICKRA_EMBED_TRAITS(Sma, WickraSma, wickra_sma)
WICKRA_EMBED_TRAITS(Rsi, WickraRsi, wickra_rsi)
WICKRA_EMBED_TRAITS(Roc, WickraRoc, wickra_roc)
WICKRA_EMBED_TRAITS(Atr, WickraAtr, wickra_atr)

#undef WICKRA_EMBED_TRAITS

/// Simple moving average over a fixed window.
class Sma : public Handle<SmaTraits> {
public:
    Update update(double input) {
        double out = 0.0;
        int s = ok() ? wickra_sma_update(raw(), input, &out) : WICKRA_EMBED_ERR_NULL;
        return finish(s, out);
    }
};

/// The EMA is the one indicator initialised with a run-time period, so its
/// traits carry the period and its constructor takes one.
struct EmaTraits {
    using Raw = WickraEma;
    static std::size_t size() { return static_cast<std::size_t>(wickra_ema_size()); }
    static std::size_t align() { return static_cast<std::size_t>(wickra_ema_align()); }
    static int init(Raw* h) { return wickra_ema_init(h, 20); }
    static void reset(Raw* h) { wickra_ema_reset(h); }
    static uintptr_t warmup(const Raw* h) { return wickra_ema_warmup(h); }
    static int is_ready(const Raw* h) { return wickra_ema_is_ready(h); }
};

/// Exponential moving average. `Ema()` is EMA(20); `Ema(period)` any period,
/// `ok()` false for a period of 0 (`WICKRA_EMBED_ERR_PERIOD`).
class Ema : public Handle<EmaTraits> {
public:
    Ema() = default;
    explicit Ema(uintptr_t period) {
        if (status_ == WICKRA_EMBED_OK || status_ == WICKRA_EMBED_ERR_PERIOD) {
            status_ = wickra_ema_init(raw(), period);
        }
    }
    Update update(double input) {
        double out = 0.0;
        int s = ok() ? wickra_ema_update(raw(), input, &out) : WICKRA_EMBED_ERR_NULL;
        return finish(s, out);
    }
};

/// Relative strength index.
class Rsi : public Handle<RsiTraits> {
public:
    Update update(double input) {
        double out = 0.0;
        int s = ok() ? wickra_rsi_update(raw(), input, &out) : WICKRA_EMBED_ERR_NULL;
        return finish(s, out);
    }
};

/// Rate of change.
class Roc : public Handle<RocTraits> {
public:
    Update update(double input) {
        double out = 0.0;
        int s = ok() ? wickra_roc_update(raw(), input, &out) : WICKRA_EMBED_ERR_NULL;
        return finish(s, out);
    }
};

/// Average true range over a candle's open, high, low and close.
class Atr : public Handle<AtrTraits> {
public:
    Update update(double open, double high, double low, double close) {
        double out = 0.0;
        int s = ok() ? wickra_atr_update(raw(), open, high, low, close, &out) : WICKRA_EMBED_ERR_NULL;
        return finish(s, out);
    }
};

/// The library version string, e.g. "0.1.0".
inline const char* version() { return wickra_embed_version(); }

}  // namespace wickra_embed

#endif  // WICKRA_EMBED_HPP
