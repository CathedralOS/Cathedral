#!/usr/bin/env bash
# Compile and inspect Cathedral's detached paging geometry and exact-zero
# page-table validator without claiming storage, mappings, or installation.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CANARY_ROOT="$REPO_ROOT/tools/x86-empty-page-table-canary"
OMEGA_REPO="${OMEGA_REPO:-$REPO_ROOT/../Omega}"
SCRATCH_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cathedral-x86-empty-page-table.XXXXXX")"
PROJECT_DIR="$SCRATCH_DIR/project"
BUILD_DIR="$SCRATCH_DIR/artifacts"
trap 'rm -rf "$SCRATCH_DIR"' EXIT

mkdir -p "$PROJECT_DIR"
install -m 0644 "$CANARY_ROOT/main.omg" "$PROJECT_DIR/main.omg"
install -m 0644 "$CANARY_ROOT/build.omg" "$PROJECT_DIR/build.omg"
ln -s "$REPO_ROOT/source/core" "$PROJECT_DIR/core"
ln -s "$REPO_ROOT/source/drivers/facts" "$PROJECT_DIR/facts"
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
  echo "error: jq is required to validate empty page-table artifacts" >&2
  exit 2
fi

run_omega --check --build-dir "$BUILD_DIR" "$CANARY_MAIN"

TYPED="$BUILD_DIR/04_typed_trees.json"
CONTRACTS="$BUILD_DIR/05_machine_contracts.json"
ASSERTIONS="$CANARY_ROOT/assert-empty.jq"
WALK_ASSERTIONS="$CANARY_ROOT/assert-walk.jq"
ROLE_ASSERTIONS="$CANARY_ROOT/assert-roles.jq"

for artifact in "$TYPED" "$CONTRACTS" "$ASSERTIONS" "$WALK_ASSERTIONS" "$ROLE_ASSERTIONS"; do
  [[ -f "$artifact" ]] || {
    echo "error: expected compiler artifact is missing: $artifact" >&2
    exit 1
  }
done

jq -s -e -f "$ASSERTIONS" "$TYPED" "$CONTRACTS" >/dev/null
jq -s -e -f "$WALK_ASSERTIONS" "$TYPED" "$CONTRACTS" >/dev/null
jq -s -e -f "$ROLE_ASSERTIONS" "$TYPED" "$CONTRACTS" >/dev/null

echo "Cathedral x86 empty-page-table canary passed"
