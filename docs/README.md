# Documentation

The reference documentation for Wickra Embed lives at
**[embed.wickra.org](https://embed.wickra.org)** — the no-alloc core, the C ABI
handle contract and the target guides.

What stays here, beside the code, is the material that only makes sense next to
the implementation and has to change in the same commit as it:

- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — the layers and the no-alloc design.
- [`NO_STD.md`](NO_STD.md) — no_std / no-alloc design, ports, panic handler, targets.
- [`INDICATORS.md`](INDICATORS.md) — the v0.1 subset and how to extend it.
- [`PARITY.md`](PARITY.md) — the byte-parity moat against `wickra-core`.
- [`C_ABI.md`](C_ABI.md) — the no-alloc C ABI handle contract.
- [`LATENCY.md`](LATENCY.md) — bounded per-update latency and how it is measured.
