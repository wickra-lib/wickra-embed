# Wickra Embed — C / C++ examples

A heap-free SMA(20) crossover signal driven through the wickra-embed C ABI. The
indicator handle lives in a **stack buffer the caller provides** — there is no
`malloc` and no `free`. Because the handle size depends on the target (pointer
width, window length), the ABI reports it at runtime (`wickra_sma_size()`), and
the example asserts the handle fits its buffer before using it — the same pattern
firmware uses to place the handle in a static or stack allocation. The C++
example runs the same signal through the header-only wrapper in
[`wickra_embed.hpp`](../../bindings/c/include/wickra_embed.hpp).

## Build the library

From the workspace root:

```sh
cargo build -p wickra-embed-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra_embed.so`     | `-lwickra_embed` |
| macOS    | `libwickra_embed.dylib`  | `-lwickra_embed` |
| Windows (MSVC) | `wickra_embed.dll` | `wickra_embed.dll.lib` (import lib) |

A static library (`libwickra_embed.a` / `wickra_embed.lib`) is emitted alongside.

## Build and run the examples

### With CMake (portable, used by CI)

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

On Windows the build copies `wickra_embed.dll` next to the executable so the
loader finds it; on Linux/macOS the `.so`/`.dylib` is resolved via the embedded
library path. Override `WICKRA_EMBED_LIB_DIR` for an out-of-tree library
location.

### Directly with a compiler

```sh
# Linux / macOS
cc examples/c/sma_signal.c -I bindings/c/include -L target/release -lwickra_embed -lm -o sma_signal
LD_LIBRARY_PATH=target/release ./sma_signal        # macOS: DYLD_LIBRARY_PATH

# Windows (MinGW gcc, linking the DLL directly)
gcc examples/c/sma_signal.c -I bindings/c/include target/release/wickra_embed.dll -lm -o sma_signal.exe
```

The C++ example is the same command with `c++`/`g++` and `sma_signal.cpp`; the
header-only wrapper needs no extra library.

## The examples

| Example | What it does |
|---------|--------------|
| `sma_signal.c` | The crossover signal with zero heap, over the C ABI. |
| `sma_signal.cpp` | The same signal through the header-only C++ wrapper. |

For each warm bar (after the 20-input warmup) both print the price, the moving
average, and whether the price is `ABOVE` or `below` it — a minimal trading
signal computed with zero allocations. The final line confirms the warm-bar
count matches `60 - warmup`.

## Usage shape

Every indicator follows the same no-alloc pattern: ask the ABI for the handle
size, place the handle in storage you own, `init` it there, feed it inputs, and
never free anything — the storage is yours. The full C ABI — the handle
contract, every function and the return codes — is declared in the header:
[`bindings/c/include/wickra_embed.h`](../../bindings/c/include/wickra_embed.h),
and [`bindings/c/README.md`](../../bindings/c/README.md) walks through it.
