---
name: Bug report
about: Report incorrect behaviour or a crash
title: "[bug] "
labels: bug
---

**Describe the bug**
A clear description of what is wrong.

**Reproduction**
The smallest input that reproduces the problem — an indicator, a period, and the
sequence of prices or candles fed to `update`.

```
# paste a minimal repro here (indicator + params + a few input values)
```

**Expected vs actual**
- Expected: … (ideally the `wickra-core` value it should match)
- Actual: …

**Environment**
- `wickra-embed` version:
- Rust toolchain (`rustc --version`):
- Target (`x86_64-*` host / `thumbv7em-none-eabihf` / `thumbv6m-none-eabi`):
- OS:

**Additional context**
Anything else (logs, the parity delta, whether it reproduces on host and on the MCU target).
