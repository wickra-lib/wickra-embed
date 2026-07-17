# C ABI example — `sma_signal`

A heap-free SMA(20) crossover signal driven through the wickra-embed C ABI. The
indicator handle lives in a **stack buffer the caller provides** — there is no
`malloc` and no `free`. Because the handle size depends on the target (pointer
width, window length), the ABI reports it at runtime (`wickra_sma_size()`), and
the example asserts the handle fits its buffer before using it — the same pattern
firmware uses to place the handle in a static or stack allocation.

## Build & run

```bash
# 1. Build the C ABI library (cdylib + staticlib).
cargo build --release -p wickra-embed-c

# 2. Configure and build the example.
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release

# 3. Run it (via ctest, or directly).
ctest --test-dir examples/c/build -C Release --output-on-failure
```

On Windows the build copies `wickra_embed.dll` next to the executable so the
loader finds it; on Linux/macOS the `.so`/`.dylib` is resolved via the embedded
library path. Override `WICKRA_EMBED_LIB_DIR` for an out-of-tree library
location.

## What it prints

For each warm bar (after the 20-input warmup) it prints the price, the moving
average, and whether the price is `ABOVE` or `below` it — a minimal trading
signal computed with zero allocations. The final line confirms the warm-bar
count matches `60 - warmup`.

The full C ABI — the handle contract, every function, and the return codes — is
declared in the header:
[`bindings/c/include/wickra_embed.h`](../../bindings/c/include/wickra_embed.h).
