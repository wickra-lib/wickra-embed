// wickra-embed C++ example: the same crossover signal as sma_signal.c, through
// the header-only wrapper in wickra_embed.hpp. Still zero heap: the handle's
// storage is part of the `wickra_embed::Sma` object on the stack, and the
// wrapper checks the run-time handle size against its capacity in the
// constructor instead of trusting a hand-picked buffer.
#include <cstdio>

#include "wickra_embed.hpp"

static double price(int i) {
    return 100.0 + 8.0 * ((double) (i % 12) - 6.0) / 6.0 + 0.1 * (double) i;
}

int main() {
    std::printf("wickra-embed %s\n", wickra_embed::version());

    wickra_embed::Sma sma;
    if (!sma.ok()) {
        std::fprintf(stderr, "the SMA handle did not fit its storage\n");
        return 1;
    }
    std::printf("warmup: %zu inputs\n", sma.warmup());

    int warm_bars = 0;
    for (int i = 0; i < 60; i++) {
        const double p = price(i);
        const wickra_embed::Update r = sma.update(p);
        if (r.status < 0) {
            std::fprintf(stderr, "update failed: %d\n", r.status);
            return 1;
        }
        if (r.ready) {
            warm_bars++;
            std::printf("bar %2d  price %8.3f  sma20 %8.3f  %s\n", i, p, r.value,
                        p > r.value ? "above" : "below");
        }
    }
    // SMA(20) emits its first value on the 20th input: 60 bars -> 41 warm ones.
    if (warm_bars != 41 || !sma.is_ready()) {
        std::fprintf(stderr, "expected 41 warm bars, got %d\n", warm_bars);
        return 1;
    }
    sma.reset();
    if (sma.is_ready()) {
        std::fprintf(stderr, "reset did not clear the state\n");
        return 1;
    }
    std::printf("ok\n");
    return 0;
}
