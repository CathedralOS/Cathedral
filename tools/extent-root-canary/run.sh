#!/usr/bin/env bash
# Compile Cathedral's checked memory-root provider without booting QEMU or
# executing any firmware/hardware operation.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CANARY_ROOT="$REPO_ROOT/tools/extent-root-canary"
OMEGA_REPO="${OMEGA_REPO:-$REPO_ROOT/../Omega}"
SCRATCH_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cathedral-extent-root.XXXXXX")"
PROJECT_DIR="$SCRATCH_DIR/project"
BUILD_DIR="$SCRATCH_DIR/artifacts"
trap 'rm -rf "$SCRATCH_DIR"' EXIT

# Keep provider-plan lock refreshes and all compiler output out of the source
# tree. The checked Cathedral core is linked into an isolated canary frontier.
mkdir -p "$PROJECT_DIR"
install -m 0644 "$CANARY_ROOT/main.omg" "$PROJECT_DIR/main.omg"
install -m 0644 "$CANARY_ROOT/build.omg" "$PROJECT_DIR/build.omg"
ln -s "$REPO_ROOT/source/core" "$PROJECT_DIR/core"
CANARY_MAIN="$PROJECT_DIR/main.omg"

run_omega() {
  if [[ -n "${OMEGA_BIN:-}" && -x "${OMEGA_BIN:-}" ]]; then
    "$OMEGA_BIN" "$@"
  elif command -v omega >/dev/null 2>&1; then
    omega "$@"
  elif [[ -x "$OMEGA_REPO/target/debug/omega" ]]; then
    "$OMEGA_REPO/target/debug/omega" "$@"
  elif command -v cargo >/dev/null 2>&1 && [[ -f "$OMEGA_REPO/Cargo.toml" ]]; then
    cargo run -q --manifest-path "$OMEGA_REPO/Cargo.toml" -p omega-cli -- "$@"
  else
    echo "error: no 'omega' toolchain or sibling Omega workspace" >&2
    exit 2
  fi
}

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required to validate qualified extent-root artifacts" >&2
  exit 2
fi

run_omega --check --build-dir "$BUILD_DIR" "$CANARY_MAIN"

QUALIFICATION="$BUILD_DIR/05_qualification_evidence.json"
OUTCOMES="$BUILD_DIR/05_claim_outcomes.json"
CONTRACTS="$BUILD_DIR/05_machine_contracts.json"
TYPED="$BUILD_DIR/04_typed_trees.json"
TRUST_REPORT="$BUILD_DIR/trust_report.md"
ROOT_ASSERTIONS="$CANARY_ROOT/assert-root.jq"

for artifact in \
  "$QUALIFICATION" \
  "$OUTCOMES" \
  "$CONTRACTS" \
  "$TYPED" \
  "$TRUST_REPORT" \
  "$ROOT_ASSERTIONS"; do
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

# Pin the build-owned admission and selection rather than treating a checked
# adapter body or a same-named type as authority.
assert_contains "$TYPED" '"target": "accept_boundary#ExtentRootProvider"'
assert_contains "$TYPED" '"target": "select_provider#ExtentRootProvider#CathedralExtentRootProvider"'
assert_contains "$TRUST_REPORT" 'provider plan: CathedralExtentRootProvider::satisfies::ExtentRootProvider'
assert_contains "$TRUST_REPORT" 'coverage 1/1 -- root grant (build.omg)'
assert_contains "$TRUST_REPORT" 'accepted fact: ExtentRootProvider -- root grant (build.omg)'

jq -s -e -f "$ROOT_ASSERTIONS" \
  "$QUALIFICATION" "$OUTCOMES" "$CONTRACTS" >/dev/null

echo "Cathedral qualified extent-root canary passed"
