#!/usr/bin/env bash
# Compile Cathedral's first timer-root contract and pin the public facts that
# later IDT/root installation must consume. This does not boot QEMU or perform
# any privileged operation.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CANARY_ROOT="$REPO_ROOT/tools/legacy-timer-root-canary"
OMEGA_REPO="${OMEGA_REPO:-$REPO_ROOT/../Omega}"
SCRATCH_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cathedral-timer-root.XXXXXX")"
PROJECT_DIR="$SCRATCH_DIR/project"
BUILD_DIR="$SCRATCH_DIR/artifacts"
trap 'rm -rf "$SCRATCH_DIR"' EXIT

# Importing the concrete PIC provider makes its privileged `out` instructions
# part of this compile frontier. Build an isolated freestanding canary project
# in scratch space; the committed Cathedral package remains untouched and the
# harness still installs or executes nothing.
mkdir -p "$PROJECT_DIR"
install -m 0644 "$CANARY_ROOT/main.omg" "$PROJECT_DIR/main.omg"
ln -s "$REPO_ROOT/source/core" "$PROJECT_DIR/core"
printf '%s\n' \
  'machine build(b: &mut Build) {' \
  '    b.depend("core", path("core"));' \
  '    b.freestanding = true;' \
  '}' > "$PROJECT_DIR/build.omg"
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

run_omega --check --build-dir "$BUILD_DIR" "$CANARY_MAIN"
TERMINAL_SUMMARY="$BUILD_DIR/terminal-summary.txt"
run_omega inspect-terminal --machine LegacyPicTimerRoot::enter "$CANARY_MAIN" \
  > "$TERMINAL_SUMMARY"

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

assert_ordered_lines() {
  local artifact="$1"
  shift
  local previous=0
  local text
  local line
  for text in "$@"; do
    line="$(grep -n -F -m 1 -- "$text" "$artifact" | cut -d: -f1 || true)"
    if [[ -z "$line" ]]; then
      echo "error: $(basename "$artifact") does not contain ordered item: $text" >&2
      exit 1
    fi
    if (( line <= previous )); then
      echo "error: $(basename "$artifact") has out-of-order item: $text" >&2
      exit 1
    fi
    previous="$line"
  done
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
# root body. The selected legacy provider must delegate the admitted linear
# occurrence to the checked PIC body that executes the master EOI before
# normalized settlement. Adding scheduler or registration work to the hard
# root fails this coarse public-contract pin before fixed-fuel composition.
assert_contains "$CONTRACTS" '"machine": "LegacyPicTimerRoot::enter"'
assert_contains "$CONTRACTS" '"checked_may_suspend": false'
assert_contains "$CONTRACTS" '"checked_may_block": false'
assert_contains "$CONTRACTS" '"checked_service_reach": ["PortIo"]'
assert_contains "$CONTRACTS" '"machine": "Pic8259::complete_timer_acknowledgement"'
assert_contains "$REPO_ROOT/source/core/legacy_timer_root.omg" 'Pic8259::complete_timer_acknowledgement(acknowledgement);'
assert_contains "$SYNTAX" 'Pic8259::complete_timer_acknowledgement'
assert_contains "$SYNTAX" 'OCW2_END_OF_INTERRUPT'
assert_contains "$CONTRACTS" '{"state": "complete_timer_acknowledgement", "statement_ordinal": 1, "call_ordinal": 0, "target_machine": "InterruptAcknowledgement::complete"'

# The independently validated terminal summary is the acceptance surface for
# the hard-root executable closure. It must preserve the exact checked
# root-to-PIC transfer, symbol-backed PortIo operation, acknowledgement
# settlement, and value-less return without rebinding ProgramEntry.
assert_contains "$TERMINAL_SUMMARY" 'terminal selected_machine=LegacyPicTimerRoot::enter entry=machine:1'
assert_contains "$TERMINAL_SUMMARY" 'kind=CallUnit callee=machine:2 callee_attachment=named(name(Pic8259))'
assert_contains "$TERMINAL_SUMMARY" 'transfers=[claim:1->argument:0]'
assert_contains "$TERMINAL_SUMMARY" 'kind=PortWrite service=service:1 service_identity=PortIo port=0x0020 value=0x20'
assert_contains "$TERMINAL_SUMMARY" 'kind=BoundaryCallUnit boundary=boundary:1 boundary_identity=InterruptAcknowledgement::complete'
assert_contains "$TERMINAL_SUMMARY" 'settlements=[claim:2->argument:0]'
assert_ordered_lines "$TERMINAL_SUMMARY" \
  'machine id=machine:1 attachment=named(name(LegacyPicTimerRoot)) result=unit' \
  'kind=CallUnit callee=machine:2 callee_attachment=named(name(Pic8259))' \
  'machine id=machine:2 attachment=named(name(Pic8259)) result=unit' \
  'kind=PortWrite service=service:1 service_identity=PortIo port=0x0020 value=0x20' \
  'kind=BoundaryCallUnit boundary=boundary:1 boundary_identity=InterruptAcknowledgement::complete' \
  'terminator machine=machine:2 block=block:2 kind=ReturnUnit edge=edge:2'

echo "Cathedral legacy timer-root canary passed"
