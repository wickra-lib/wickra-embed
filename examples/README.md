# Wickra Embed examples

Three ways to run the allocation-free `wickra-embed-core` indicators, from a desktop
sanity check down to bare metal. Each is its own workspace (or a CMake project) so
the repo-root `cargo` commands never pull `std` onto the `no_std` core. The host
and embedded examples share the same indicator code — only the `std`/`no_std`
boundary and the I/O differ, which is the whole point: the value on the desktop
equals the value on the chip.

## Rust — `examples/host/`, `examples/embedded/`

| Example | What it shows | How to run |
|---------|---------------|------------|
| [`host/`](host/src/main.rs) | `std` sanity run: reads the golden price vector and prints `SMA(20)`/`EMA(20)`. | `cargo run --manifest-path examples/host/Cargo.toml` |
| [`embedded/`](embedded/README.md) | Bare-metal **Cortex-M** under QEMU: no OS, no heap, DWT cycle timing. | See [`embedded/README.md`](embedded/README.md) |

## C / C++ — `examples/c/`

The no-alloc **C ABI** with a caller-provided stack handle — no `malloc`, no
`free`. Build the library first (`cargo build -p wickra-embed-c --release`), then
build and run the examples via CMake, as the CI C ABI job does:

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

| Example | What it does |
| --- | --- |
| `sma_signal.c` | A crossover signal with zero heap, over the C ABI. |
| `sma_signal.cpp` | The same crossover signal as `sma_signal.c`, through the header-only wrapper in `wickra_embed.hpp`. |

[`c/README.md`](c/README.md) has the build matrix and the direct-compiler path.

## Example datasets

The host and C examples read the golden price vector; the cross-language golden
fixtures, which the C ABI and the core are checked against byte for byte, live in
[`../golden/`](../golden).
