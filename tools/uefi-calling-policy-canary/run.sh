#!/usr/bin/env bash
# Compile Cathedral's UEFI policy contract without loading the boot package's
# provider grants or executing QEMU.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CANARY_ROOT="$REPO_ROOT/tools/uefi-calling-policy-canary"
OMEGA_REPO="${OMEGA_REPO:-$REPO_ROOT/../Omega}"
BUILD_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cathedral-uefi-policy.XXXXXX")"
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

SYNTAX="$BUILD_DIR/02_syntax_trees.json"
TYPED="$BUILD_DIR/04_typed_trees.json"
CONTRACTS="$BUILD_DIR/05_machine_contracts.json"
POLICY_ASSERTIONS="$CANARY_ROOT/assert-policy.jq"

for artifact in "$SYNTAX" "$TYPED" "$CONTRACTS" "$POLICY_ASSERTIONS"; do
  [[ -f "$artifact" ]] || {
    echo "error: expected compiler artifact is missing: $artifact" >&2
    exit 1
  }
done

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required to validate the typed UEFI calling policy" >&2
  exit 2
fi

assert_contains() {
  local artifact="$1"
  local text="$2"
  if ! grep -F -q -- "$text" "$artifact"; then
    echo "error: $(basename "$artifact") does not contain: $text" >&2
    exit 1
  fi
}

# The evidence identity is deliberate and must not regress to the retired
# unnamed `Subject satisfies Trait` declaration.
assert_contains "$SYNTAX" '"text": "UefiX86_64CallingPolicy"'
assert_contains "$TYPED" '"name": "UefiX86_64::plan"'
assert_contains "$TYPED" '"name": "UefiApplication"'
assert_contains "$TYPED" 'Calling<UefiX86_64>'

# Isolate the exact typed policy machine and its matching checked contract, then
# validate Cathedral's actual firmware entry plan and pure terminating body.
# Merely finding these vocabulary names elsewhere in either artifact would not
# prove that UefiX86_64::plan selected or established them.
jq -s -e -f "$POLICY_ASSERTIONS" "$TYPED" "$CONTRACTS" >/dev/null

echo "Cathedral UEFI calling-policy canary passed"
