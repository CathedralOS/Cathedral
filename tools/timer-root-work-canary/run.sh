#!/usr/bin/env bash
# Execute the timer wake handoff in both Omega engines.
set -euo pipefail

CANARY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OMEGA_BIN="${OMEGA_BIN:-$(command -v omega || true)}"
if [[ -z "$OMEGA_BIN" || ! -x "$OMEGA_BIN" ]]; then
    echo "error: set OMEGA_BIN to an optimized Omega compiler, or install omega on PATH" >&2
    exit 2
fi

"$OMEGA_BIN" update --project "$CANARY_ROOT"
"$OMEGA_BIN" run --both "$CANARY_ROOT/main.omg"
echo "Cathedral timer wake publication and consumption passed"
