# Bounded integer AML execution checks

These tests consume real AML bytes through `cathedral-acpi-execution`. The
79 scenarios cover translated method/loop/increment/logical-not examples,
integer stores/calls/control flow, malformed inputs and explicit limits/services.
The [port record](../../../../../source/libraries/acpi/interpreter/execution/PORT.md)
describes exact scope and pending generic interpreter behavior.

```sh
python3 tools/ports/acpi/interpreter/execution/fixtures.py --check
python3 tools/ports/acpi/interpreter/execution/evidence.py --check
python3 tools/ports/acpi/interpreter/execution/verify_record.py
python3 tools/ports/acpi/interpreter/execution/check_interpreted.py \
  --record tools/ports/acpi/interpreter/execution/verification.json
```

`check_interpreted.py` builds a Cathedral-owned Rust test consumer against the
clean sibling Omega source, using its pinned Rust toolchain. Dependencies are
resolved offline with the retained `runner.Cargo.lock` and `--locked`. It uses
the package manager's ordinary source inspection
pipeline, retains the checked authored bodies and calls
`checked_interpreter::interpret_entry` on each selected case. Every positive must
return 0 and every changed-body expected-value control must return 1 without an
interpreter error. Controls change the actual result/outcome comparison inside
the Omega test, so a default-zero result cannot pass. No Python model supplies
execution results. `--match` accepts comma-separated name substrings for focused
runs; the default checks the full corpus.

The harness evaluator has a fixed 10,000,000-step ceiling. This is separate from
and does not change the implemented AML caller budget (at most 1024 execution
turns, shared by loops and calls), expression/block/frame limits or input bounds.
The runner prints the exact clean Omega revision and Rust harness binary hash.
The Rust helper selects no native publication or live host effects.

A current frame-admission constant proof includes the maximum-offset regression:

```sh
python3 tools/ports/acpi/interpreter/execution/check_frame_const.py
```

A separate, slower bytecode constant-evaluation proof remains reproducible:

```sh
python3 tools/ports/acpi/interpreter/execution/check.py \
  --omega /tmp/cathedral-omega-eaa7993/release/omega \
  --match return_add,while_increment --jobs 1
```

`check.py` requires `const TEST_RESULT = test_result()` to satisfy a checked
zero-result requirement. Its mutated body must fail with an actual `1 == 0`
requirement diagnostic. The compiler's smaller 100,000 const-evaluator step cap
can reject more complex scenarios even when the executor's own fuel is within
profile. Such a rejection is not a passing bytecode test.

Fixtures hand-encode AML from the primary grammar and seed the parser's public
namespace/method-span representation. They do not run iASL, load complete ASL
DefinitionBlocks, compare with an external interpreter, emit native code, access
hardware or establish production integration. All 79 checked-interpreter scenarios and 79 changed-body controls pass.
`verification.json` binds their outcomes to the current source hashes and runner;
`verify_record.py` validates the retained evidence. The full run took 214.096s.
The separate current frame-admission const proof and changed-body control also
pass, including maximum-start rejection. The optional whole-bytecode const
command is distinct from this observed final const result.
