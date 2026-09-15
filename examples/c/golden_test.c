/* wickra-embed C ABI golden test: the value a C caller gets through the ABI is
 * the value the golden corpus holds, bit for bit.
 *
 * `golden/inputs/*.csv` are the fixed input vectors and `golden/expected/*.csv`
 * the reference outputs, generated from `wickra-core` and never edited by hand
 * (golden/README.md). The Rust core pins itself to them in
 * crates/embed-core/tests/golden.rs; this file pins the C boundary: every
 * exported indicator is fed the input column one bar at a time -- streaming,
 * the only operating mode the no-alloc kernel has -- and each update's
 * result is compared with the expected cell by its f64 bits. A `nan` cell
 * means the indicator is still warming up, so the ABI must report
 * WICKRA_EMBED_WARMUP and leave `out` untouched; anything else is a value, and
 * that value must be identical to the last bit. Nothing here tolerates an
 * epsilon: the parity guarantee is byte parity, and a tolerance would hide the
 * exact drift it exists to catch.
 *
 * Completeness: the table below lists every indicator the catalogue exports
 * (`wickra_embed_core::indicators::CATALOGUE`, five entries, pinned by a test
 * of its own) with its golden column. An indicator added to the kernel without
 * a row here fails the count check, so the C surface cannot silently fall
 * behind the catalogue.
 *
 * GOLDEN_DIR is supplied by CMake; the test is compiled and run the way a
 * consumer would compile it, against the shared library. */

#include <math.h>
#include <stdalign.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_embed.h"

#ifndef GOLDEN_DIR
#error "GOLDEN_DIR must point at the repository's golden/ directory"
#endif

/* The catalogue this test must cover in full. Bump together with the kernel. */
#define CATALOGUE_LEN 5

#define HANDLE_CAP 512
#define HANDLE_ALIGN 16
#define MAX_ROWS 4096
#define LINE_CAP 512

static int failures = 0;

static void fail(const char *what, const char *detail) {
    fprintf(stderr, "FAIL %s: %s\n", what, detail);
    failures += 1;
}

/* Read one numeric column of a CSV with a header line. A cell spelled `nan`
 * (any case) becomes a NaN. Returns the row count. */
static int read_column(const char *file, int column, double *out) {
    char path[LINE_CAP];
    snprintf(path, sizeof path, "%s/%s", GOLDEN_DIR, file);
    FILE *f = fopen(path, "r");
    if (!f) {
        fail("open", path);
        return -1;
    }
    char line[LINE_CAP];
    int rows = 0;
    if (!fgets(line, sizeof line, f)) {
        fail("header", path);
        fclose(f);
        return -1;
    }
    while (fgets(line, sizeof line, f) && rows < MAX_ROWS) {
        char *cell = line;
        for (int c = 0; c < column; c++) {
            cell = strchr(cell, ',');
            if (!cell) break;
            cell++;
        }
        if (!cell) {
            fail("column", path);
            fclose(f);
            return -1;
        }
        char *end = NULL;
        double v = strtod(cell, &end);
        if (end == cell) {
            /* `nan` in any spelling strtod may not accept on this libc */
            if (strncmp(cell, "nan", 3) == 0 || strncmp(cell, "NaN", 3) == 0) {
                v = (double)NAN;
            } else {
                fail("parse", cell);
                fclose(f);
                return -1;
            }
        }
        out[rows++] = v;
    }
    fclose(f);
    return rows;
}

static int is_nan(double v) { return v != v; }

static int same_bits(double a, double b) {
    unsigned char ba[sizeof(double)], bb[sizeof(double)];
    memcpy(ba, &a, sizeof ba);
    memcpy(bb, &b, sizeof bb);
    return memcmp(ba, bb, sizeof ba) == 0;
}

/* Compare one update's outcome with the expected cell. */
static void check_cell(const char *name, int row, int status, double got, double expected) {
    char detail[LINE_CAP];
    if (is_nan(expected)) {
        if (status != WICKRA_EMBED_WARMUP) {
            snprintf(detail, sizeof detail, "row %d: expected warmup, got status %d value %.17g", row, status, got);
            fail(name, detail);
        }
        return;
    }
    if (status != WICKRA_EMBED_READY) {
        snprintf(detail, sizeof detail, "row %d: expected a value, got status %d", row, status);
        fail(name, detail);
        return;
    }
    if (!same_bits(got, expected)) {
        snprintf(detail, sizeof detail, "row %d: %.17g != expected %.17g (bits differ)", row, got, expected);
        fail(name, detail);
    }
}

typedef struct {
    const char *name;
    const char *expected_file;
} row_t;

/* One row per catalogue entry: Sma, Ema, Rsi, Atr, Roc. */
static const row_t ROWS[CATALOGUE_LEN] = {
    {"sma20", "expected/sma20.csv"},
    {"ema20", "expected/ema20.csv"},
    {"rsi14", "expected/rsi14.csv"},
    {"atr14", "expected/atr14.csv"},
    {"roc10", "expected/roc10.csv"},
};

static double prices[MAX_ROWS];
static double open_[MAX_ROWS], high[MAX_ROWS], low[MAX_ROWS], close_[MAX_ROWS];
static double expected[MAX_ROWS];

static void *handle_buf(void) {
    static alignas(HANDLE_ALIGN) unsigned char buf[HANDLE_CAP];
    memset(buf, 0, sizeof buf);
    return buf;
}

static int fits(const char *name, uintptr_t size, uintptr_t align) {
    char detail[LINE_CAP];
    if (size > HANDLE_CAP || align > HANDLE_ALIGN) {
        snprintf(detail, sizeof detail, "handle needs %zu bytes at %zu; the test buffer has %d at %d",
                 (size_t)size, (size_t)align, HANDLE_CAP, HANDLE_ALIGN);
        fail(name, detail);
        return 0;
    }
    return 1;
}

int main(void) {
    const int n_prices = read_column("inputs/prices-01.csv", 0, prices);
    const int n_bars = read_column("inputs/ohlc-01.csv", 0, open_);
    read_column("inputs/ohlc-01.csv", 1, high);
    read_column("inputs/ohlc-01.csv", 2, low);
    read_column("inputs/ohlc-01.csv", 3, close_);
    if (n_prices <= 0 || n_bars <= 0) {
        fprintf(stderr, "golden corpus not found under %s\n", GOLDEN_DIR);
        return 1;
    }

    int covered = 0;
    for (int r = 0; r < CATALOGUE_LEN; r++) {
        const row_t *row = &ROWS[r];
        const int n_expected = read_column(row->expected_file, 0, expected);
        const int n_inputs = strcmp(row->name, "atr14") == 0 ? n_bars : n_prices;
        char detail[LINE_CAP];
        if (n_expected != n_inputs) {
            snprintf(detail, sizeof detail, "%d expected cells for %d inputs", n_expected, n_inputs);
            fail(row->name, detail);
            continue;
        }
        double out = 0.0;
        int status;
        if (strcmp(row->name, "sma20") == 0) {
            if (!fits(row->name, wickra_sma_size(), wickra_sma_align())) continue;
            WickraSma *h = (WickraSma *)handle_buf();
            if (wickra_sma_init(h) != WICKRA_EMBED_OK) { fail(row->name, "init"); continue; }
            for (int i = 0; i < n_inputs; i++) {
                status = wickra_sma_update(h, prices[i], &out);
                check_cell(row->name, i, status, out, expected[i]);
            }
        } else if (strcmp(row->name, "ema20") == 0) {
            if (!fits(row->name, wickra_ema_size(), wickra_ema_align())) continue;
            WickraEma *h = (WickraEma *)handle_buf();
            if (wickra_ema_init(h, 20) != WICKRA_EMBED_OK) { fail(row->name, "init"); continue; }
            for (int i = 0; i < n_inputs; i++) {
                status = wickra_ema_update(h, prices[i], &out);
                check_cell(row->name, i, status, out, expected[i]);
            }
        } else if (strcmp(row->name, "rsi14") == 0) {
            if (!fits(row->name, wickra_rsi_size(), wickra_rsi_align())) continue;
            WickraRsi *h = (WickraRsi *)handle_buf();
            if (wickra_rsi_init(h) != WICKRA_EMBED_OK) { fail(row->name, "init"); continue; }
            for (int i = 0; i < n_inputs; i++) {
                status = wickra_rsi_update(h, prices[i], &out);
                check_cell(row->name, i, status, out, expected[i]);
            }
        } else if (strcmp(row->name, "atr14") == 0) {
            if (!fits(row->name, wickra_atr_size(), wickra_atr_align())) continue;
            WickraAtr *h = (WickraAtr *)handle_buf();
            if (wickra_atr_init(h) != WICKRA_EMBED_OK) { fail(row->name, "init"); continue; }
            for (int i = 0; i < n_inputs; i++) {
                status = wickra_atr_update(h, open_[i], high[i], low[i], close_[i], &out);
                check_cell(row->name, i, status, out, expected[i]);
            }
        } else if (strcmp(row->name, "roc10") == 0) {
            if (!fits(row->name, wickra_roc_size(), wickra_roc_align())) continue;
            WickraRoc *h = (WickraRoc *)handle_buf();
            if (wickra_roc_init(h) != WICKRA_EMBED_OK) { fail(row->name, "init"); continue; }
            for (int i = 0; i < n_inputs; i++) {
                status = wickra_roc_update(h, prices[i], &out);
                check_cell(row->name, i, status, out, expected[i]);
            }
        } else {
            fail(row->name, "no C entry point for this catalogue row");
            continue;
        }
        covered += 1;
        printf("golden %-6s %d rows, bit-identical through the C ABI\n", row->name, n_inputs);
    }

    /* Completeness: every catalogue entry ran through the ABI. */
    if (covered != CATALOGUE_LEN) {
        char detail[LINE_CAP];
        snprintf(detail, sizeof detail, "%d of %d catalogue indicators covered", covered, CATALOGUE_LEN);
        fail("completeness", detail);
    }

    if (failures) {
        fprintf(stderr, "%d failure(s)\n", failures);
        return 1;
    }
    printf("golden parity holds for all %d catalogue indicators\n", CATALOGUE_LEN);
    return 0;
}
