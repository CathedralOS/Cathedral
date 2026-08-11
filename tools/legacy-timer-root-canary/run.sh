#!/usr/bin/env bash
# Compile Cathedral's first timer-root contract and pin the public facts that
# later IDT/root installation must consume. This does not boot QEMU or perform
# any privileged operation.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CANARY_ROOT="$REPO_ROOT/tools/legacy-timer-root-canary"
OMEGA_REPO="${OMEGA_REPO:-$REPO_ROOT/../Omega}"
BUILD_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cathedral-timer-root.XXXXXX")"
trap 'rm -rf "$BUILD_DIR"' EXIT

if [[ -n "${OMEGA_BIN:-}" && -x "${OMEGA_BIN:-}" ]]; then
  "$OMEGA_BIN" --check --build-dir "$BUILD_DIR" "$CANARY_ROOT/main.omg"
elif command -v omega >/dev/null 2>&1; then
  omega --check --build-dir "$BUILD_DIR" "$CANARY_ROOT/main.omg"
elif [[ -x "$OMEGA_REPO/target/debug/omega" ]]; then
  "$OMEGA_REPO/target/debug/omega" --check --build-dir "$BUILD_DIR" "$CANARY_ROOT/main.omg"
elif command -v cargo >/dev/null 2>&1 && [[ -f "$OMEGA_REPO/Cargo.toml" ]]; then
  cargo run -q --manifest-path "$OMEGA_REPO/Cargo.toml" -p omega-cli -- \
    --check --build-dir "$BUILD_DIR" "$CANARY_ROOT/main.omg"
else
  echo "error: no 'omega' toolchain or sibling Omega workspace" >&2
  exit 2
fi

QUALIFICATION="$BUILD_DIR/05_qualification_evidence.json"
SYNTAX="$BUILD_DIR/02_syntax_trees.json"
TYPED="$BUILD_DIR/04_typed_trees.json"
CONTRACTS="$BUILD_DIR/05_machine_contracts.json"

for artifact in "$QUALIFICATION" "$SYNTAX" "$TYPED" "$CONTRACTS"; do
  [[ -f "$artifact" ]] || {
    echo "error: expected compiler artifact is missing: $artifact" >&2
    exit 1
  }
done

assert_contains() {
  local artifact="$1"
  local text="$2"
  if ! grep -F -q -- "$text" "$artifact"; then
    echo "error: $(basename "$artifact") does not contain: $text" >&2
    exit 1
  fi
}

# Selected provider and exact accepted authority route.
assert_contains "$QUALIFICATION" '"boundary": "CathedralTimerRoot"'
assert_contains "$QUALIFICATION" '"requirement": "InterruptEntry::enter"'
assert_contains "$QUALIFICATION" '"domain": "InterruptAcknowledgement::Pending"'
assert_contains "$QUALIFICATION" '"flow": "accepts"'
assert_contains "$QUALIFICATION" '"provider_plan": "LegacyPicTimerRoot::satisfies::CathedralTimerRoot"'
assert_contains "$QUALIFICATION" '"effective_carry": {"suspension": "forbidden", "cpu": "same", "thread": "same", "address": "stable"}'

# The source policy must continue to request a deriver-owned interrupt return,
# masked preemption, and Cathedral's single-source maskable-IRQ stack class.
assert_contains "$TYPED" '"name": "CathedralTimerRoot"'
assert_contains "$TYPED" '"name": "LegacyPicTimerRoot::enter"'
assert_contains "$TYPED" '"name": "InterruptReturn"'
assert_contains "$TYPED" '"name": "Masked"'
assert_contains "$SYNTAX" 'X86_MASKABLE_IRQ_STACK'
assert_contains "$TYPED" '"type_name": "X86IstStackClass"'
assert_contains "$TYPED" '"text": "4"'

# The checked implementation remains one non-suspending, non-blocking PortIo
# root body; adding scheduler or registration work to the hard root fails this
# coarse public-contract pin before later fixed-fuel composition runs.
assert_contains "$CONTRACTS" '"machine": "LegacyPicTimerRoot::enter"'
assert_contains "$CONTRACTS" '"checked_may_suspend": false'
assert_contains "$CONTRACTS" '"checked_may_block": false'
assert_contains "$CONTRACTS" '"checked_service_reach": ["PortIo"]'

echo "Cathedral legacy timer-root canary passed"
