#!/usr/bin/env bash
# Check and lower the PIC handler, then reject missing and repeated completion.
set -euo pipefail
CANARY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OMEGA_BIN="${OMEGA_BIN:-$(command -v omega || true)}"
if [[ -z "$OMEGA_BIN" || ! -x "$OMEGA_BIN" ]]; then
    echo "error: set OMEGA_BIN to an optimized Omega compiler, or install omega on PATH" >&2
    exit 2
fi
mkdir -p "$CANARY_ROOT/build"
"$OMEGA_BIN" inspect-terminal --machine timer_entry --target uefi_x86_64 "$CANARY_ROOT/main.omg" > "$CANARY_ROOT/build/terminal.txt"
python3 "$CANARY_ROOT/check-terminal.py" "$CANARY_ROOT/build/terminal.txt"
for control in missing_ack double_ack; do
    log="$CANARY_ROOT/build/$control.txt"
    if "$OMEGA_BIN" --check --target uefi_x86_64 "$CANARY_ROOT/invalid/$control/main.omg" > "$log" 2>&1; then
        echo "error: $control unexpectedly accepted" >&2
        exit 1
    fi
    case "$control" in
        missing_ack) expected='linear value `acknowledgement` reaches scope exit without being consumed or transferred' ;;
        double_ack) expected='after `acknowledgement` was transferred or consumed' ;;
    esac
    if ! grep -F -q -- "$expected" "$log"; then
        cat "$log" >&2
        echo "error: $control rejected for an unrelated reason" >&2
        exit 1
    fi
done
echo "Cathedral PIC handler lowering and acknowledgement controls passed"
