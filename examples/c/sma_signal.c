/* wickra-embed C ABI example: a crossover signal with zero heap.
 *
 * The handle lives in a stack buffer the caller provides — there is no malloc
 * and no free anywhere. Because the handle size is target-dependent (usize
 * width, window length), the ABI exposes it at runtime via wickra_sma_size();
 * we place a generously-sized, 8-byte-aligned buffer on the stack and assert at
 * runtime that the handle fits. This is exactly how firmware would embed it in a
 * static or stack allocation.
 *
 * Feeds a short synthetic price path through SMA(20) and prints, for each warm
 * bar, whether the price is above or below its moving average. */

#include <assert.h>
#include <stdalign.h>
#include <stddef.h>
#include <stdio.h>

#include "wickra_embed.h"

/* Comfortably above any real SMA(20) handle on 32/64-bit targets; checked at
 * runtime against wickra_sma_size(). */
#define HANDLE_CAP 512

static double price(int i) {
    /* A drifting, oscillating path so the crossover actually flips. */
    return 100.0 + 8.0 * ((double) (i % 12) - 6.0) / 6.0 + 0.1 * (double) i;
}

int main(void) {
    printf("wickra-embed %s\n", wickra_embed_version());

    /* Verify the runtime handle fits our stack buffer before using it. */
    assert(wickra_sma_size() <= HANDLE_CAP);
    assert(wickra_sma_align() <= alignof(max_align_t));

    alignas(max_align_t) unsigned char storage[HANDLE_CAP];
    WickraSma *sma = (WickraSma *) storage;

    int rc = wickra_sma_init(sma);
    if (rc != WICKRA_EMBED_OK) {
        fprintf(stderr, "init failed: %d\n", rc);
        return 1;
    }

    int warm_bars = 0;
    for (int i = 0; i < 60; i++) {
        double p = price(i);
        double avg = 0.0;
        int status = wickra_sma_update(sma, p, &avg);
        if (status == WICKRA_EMBED_READY) {
            const char *signal = (p >= avg) ? "ABOVE" : "below";
            printf("bar %2d  price %8.4f  sma %8.4f  %s\n", i, p, avg, signal);
            warm_bars++;
        }
    }

    /* SMA(20) warms up for 20 inputs, so 60 - 20 = 40 warm bars. */
    printf("\n%d warm bars, warmup = %u, no heap used\n", warm_bars,
           (unsigned) wickra_sma_warmup(sma));

    assert(warm_bars == 60 - (int) wickra_sma_warmup(sma));
    return 0;
}
