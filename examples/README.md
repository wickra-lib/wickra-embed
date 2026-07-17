# Examples

Three ways to run the allocation-free `embed-core` indicators, from a desktop
sanity check down to bare metal.

| Example | What it shows | How to run |
|---------|---------------|------------|
| [`host/`](host/src/main.rs) | `std` sanity run: reads the golden price vector and prints `SMA(20)`/`EMA(20)`. | `cargo run --manifest-path examples/host/Cargo.toml` |
| [`c/`](c/README.md) | The no-alloc **C ABI** with a caller-provided stack handle — no `malloc`, no `free`. | See [`c/README.md`](c/README.md) |
| [`embedded/`](embedded/README.md) | Bare-metal **Cortex-M** under QEMU: no OS, no heap, DWT cycle timing. | See [`embedded/README.md`](embedded/README.md) |

Each is its own workspace (or a CMake project) so the repo-root `cargo` commands
never pull `std` onto the `no_std` core. The host and embedded examples share the
same indicator code — only the `std`/`no_std` boundary and the I/O differ, which
is the whole point: the value on the desktop equals the value on the chip.
