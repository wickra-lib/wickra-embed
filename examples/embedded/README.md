# Bare-metal Cortex-M example

Runs `Sma<20>` and `Ema(20)` on a `thumbv7em-none-eabihf` target with **no
operating system and no heap**, times each `update` with the DWT cycle counter,
and reports the result over semihosting. It boots under QEMU's MPS2-AN385
machine, so no physical board is required.

## Run under QEMU

Needs `qemu-system-arm` on `PATH` and the target installed
(`rustup target add thumbv7em-none-eabihf`). From this directory:

```bash
cargo run --release
```

`.cargo/config.toml` sets the default target and wires `cargo run` to
`qemu-system-arm ... -semihosting-config enable=on`, so the firmware's semihosting
output streams straight to your terminal and the program exits cleanly when done.

Expected output (cycle counts depend on the QEMU host and the FPU model, but the
warm-bar count is fixed):

```text
warm bars: 44
mean cycles/update (sma+ema): <target-dependent>
```

44 warm bars = 64 inputs − 20 (the `Ema(20)`/`Sma<20>` warmup). If you only need
to confirm it *builds* for the target without running QEMU:

```bash
cargo build --release --target thumbv7em-none-eabihf
```

## What it shows

- `embed-core` compiled with `default-features = false` — strictly `#![no_std]`,
  no allocator linked. The linker would fail if anything reached for the heap.
- Deterministic O(1) updates: the mean cycles-per-update is bounded, which is the
  property that matters for hard-real-time firmware.
- On real hardware the crossover would drive a GPIO/LED instead of a semihosting
  print; the print stands in for that so the example is observable in QEMU.

Layout lives in [`memory.x`](memory.x); the panic behaviour is `panic-halt`
(halt-on-panic, no unwinding).
