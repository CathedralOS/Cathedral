#!/usr/bin/env bash
# Evaluate Cathedral's pure UART fact snapshot without executing port I/O.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CANARY_ROOT="$REPO_ROOT/tools/uart-16550-facts-canary"
OMEGA_REPO="${OMEGA_REPO:-$REPO_ROOT/../Omega}"
SCRATCH_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cathedral-uart-16550-facts.XXXXXX")"
PROJECT_DIR="$SCRATCH_DIR/project"
trap 'rm -rf "$SCRATCH_DIR"' EXIT

mkdir -p "$PROJECT_DIR"
install -m 0644 "$CANARY_ROOT/main.omg" "$PROJECT_DIR/main.omg"
install -m 0644 "$CANARY_ROOT/build.omg" "$PROJECT_DIR/build.omg"
cp -R "$REPO_ROOT/source/drivers/facts" "$PROJECT_DIR/facts"
CANARY_MAIN="$PROJECT_DIR/main.omg"

run_omega() {
  if [[ -n "${OMEGA_BIN:-}" && -x "${OMEGA_BIN:-}" ]]; then
    "$OMEGA_BIN" "$@"
  elif command -v omega >/dev/null 2>&1; then
    omega "$@"
  elif [[ -x "$OMEGA_REPO/target/debug/omega" ]]; then
    "$OMEGA_REPO/target/debug/omega" "$@"
  elif command -v cargo >/dev/null 2>&1 && [[ -f "$OMEGA_REPO/Cargo.toml" ]]; then
    cargo run -q --manifest-path "$OMEGA_REPO/Cargo.toml" -p omega -- "$@"
  else
    echo "error: no 'omega' toolchain or sibling Omega workspace" >&2
    exit 2
  fi
}

run_omega --check "$CANARY_MAIN"

# A changed expected COM1 value must execute the same body and reject result1.
python3 - "$CANARY_MAIN" <<'PYTHON'
from pathlib import Path
import sys
p = Path(sys.argv[1])
s = p.read_text()
assert s.count('snapshot.com1 == 1016') == 1
p.write_text(s.replace('snapshot.com1 == 1016', 'snapshot.com1 == 1017'))
PYTHON
if run_omega --check "$CANARY_MAIN" >"$SCRATCH_DIR/negative.log" 2>&1; then
  echo "error: mutated UART fact expectation unexpectedly passed" >&2
  exit 1
fi
python3 - "$SCRATCH_DIR/negative.log" <<'PYTHON'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
assert 'cannot prove requires contract' in text and '1 == 0' in text, text
PYTHON

echo "Cathedral UART fact semantic canary passed; mutated expected value rejected"
