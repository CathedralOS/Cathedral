#!/usr/bin/env bash
# Current source/layout consumer check; no retired JSON dump requirement.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OMEGA_BIN="${OMEGA_BIN:-$REPO_ROOT/../Omega/target/release/omega}"
"$OMEGA_BIN" --check "$REPO_ROOT/tools/x86-idt-gate-layout-canary/main.omg"
"$OMEGA_BIN" --check "$REPO_ROOT/tools/ports/x86_64-interrupts/layout_local_projection.omg"
python3 "$REPO_ROOT/tools/ports/x86_64-interrupts/check_gate_policy.py"
echo "Cathedral IDT canonical-schema/source and local-equivalent layout checks passed"
