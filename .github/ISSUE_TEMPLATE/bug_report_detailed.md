---
name: Bug report (Detailed)
about: Long-form bug report with environment matrix, minimal reproducer, and expected-vs-actual sections.
title: "[Bug] <short description>"
labels: ["bug", "triage"]
assignees: []
---

## Summary

<!-- One or two sentences. What did you expect, what happened instead? -->

## Affected binding

- [ ] Rust crate (`wickra-embed-core`, host or `no_std`)
- [ ] C ABI staticlib (`bindings/c`, a release archive per target)
- [ ] Embedded example (`examples/embedded`, thumbv7em)
- [ ] Docs / examples only

## Environment

| Field                | Value                                  |
| -------------------- | -------------------------------------- |
| Wickra Embed version       | `e.g. 0.4.2`                           |
| Target               | `e.g. x86_64-unknown-linux-gnu, thumbv7em-none-eabihf` |
| OS / arch            | `e.g. Windows 11 x86_64, Linux glibc`  |
| Rust toolchain       | `rustc --version` (If building from source) |
| C compiler           | `gcc --version` / `arm-none-eabi-gcc --version` (C ABI consumers) |

## Minimal reproducer

<!--
Paste the smallest possible code snippet that triggers the bug.
If the input data matters, attach a CSV/JSON or paste a few rows inline.
-->

```python
# or rust / js
import wickra_embed as ta
...
```

## Actual output

```
<paste stack trace, panic, wrong values, etc.>
```

## Expected output

<!-- What should the indicator / API have returned? Reference a paper, TA-Lib, or another implementation if possible. -->

## Additional context

<!-- Logs, screenshots, links to related issues, anything else useful. -->
