#!/usr/bin/env bash
#
# Regenerate every committed lockfile in the repository:
#   - Rust:   Cargo.lock                          (cargo update)
#             examples/host/Cargo.lock            (detached workspace)
#             examples/embedded/Cargo.lock        (detached workspace)
#
# Run from anywhere; the script cd's to the repository root itself:
#
#     ./scripts/update-lockfiles.sh
#
# There are no Node or Python locks here: wickra-embed ships a no_std core and
# a C ABI, and CI's only Python is the metadata audit, which needs nothing
# installed. fuzz/Cargo.lock is deliberately not here: the fuzz crate is a
# detached workspace and its lock is gitignored, so there is nothing committed
# to refresh.
#
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "==> Rust (Cargo.lock)"
cargo update

echo "==> Rust examples (examples/host, examples/embedded)"
(cd examples/host && cargo update)
(cd examples/embedded && cargo update)

echo "done: review the diff, then commit the lockfiles."
