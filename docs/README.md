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

## Editing the docs

The documentation site is a separate git repository at
`https://github.com/wickra-lib/wickra-embed-site`. Open a pull request there to
propose changes; the site is built with VitePress and deploys to
`embed.wickra.org`. The files in this directory change in the same commit as
the code they describe.
