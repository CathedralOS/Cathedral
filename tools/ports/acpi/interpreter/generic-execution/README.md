# Canonical generic execution fixtures

Canonical verification for preloaded String/Buffer/Package transport,
method arguments and Return, generic CopyObject, Local/Arg replacement,
identity-preserving byte cells and primary argument-binding isolation.

`fixtures.py` authors initialized synthetic AML, loads it with the actual
canonical loader and runs the actual executor through Program. Cases which seed
canonical Reference or malformed storage metadata explicitly say so; they do
not claim RefOf opcode execution. Data-copy independence is tested by mutating
the source afterward through actual byte-storage field APIs. Shared argument
payload identity is tested by the complementary post-call mutation witness.
Each control changes an expected number, bytes, length or execution outcome
before evaluating the original assertion body. It does not merely change the
final assertion or returned test code.

Public host comparisons are retained independently by
`tools/ports/acpi/aml-public-execution/generic-observations.json`. In particular,
the pin mutates plain named/local incoming arguments; this profile instead uses
the primary call-by-reference-constant binding rule. Shallow Package child
sharing is an explicit bounded profile, not a full deep-clone claim.

Commands (existing isolated runner; no shared Cargo target build):

```
python3 tools/ports/acpi/interpreter/generic-execution/fixtures.py
python3 tools/ports/acpi/interpreter/generic-execution/check.py --record tools/ports/acpi/interpreter/generic-execution/verification.json
python3 tools/ports/acpi/interpreter/execution/check_interpreted.py --omega-source /Users/zcanann/Documents/projects/Omega --runner /tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner --record tools/ports/acpi/interpreter/generic-execution/integer-regression.json
python3 tools/ports/acpi/pipeline/check.py --omega-source /Users/zcanann/Documents/projects/Omega --runner /tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner --record tools/ports/acpi/interpreter/generic-execution/pipeline-regression.json
```

Canonical replay passed: eight checked-interpreter receipts cover 259
scenario/control pairs, and one representative constant-evaluator pair passes.
The integrated suites contain 55 generic, 79 original integer and 22 original
pipeline cases. Focused receipts cover 15 external-argument cases, six decoder
failures, four public-entry return-shape/value controls, fourteen full
Frame/ObjectStore atomicity cases and 64 binding-prediction parity cases.
Behavior overlaps between suites; counts do not imply distinct specification
requirements. `manifest.json` binds canonical source, fixture, root/build and
runner evidence. Historical scratch and prepublication canonical inputs are
archived separately under `history/`.
Checked-interpreter,
constant-evaluator and native execution are separate stages. No native or live
hardware execution is claimed.

The regression drivers' `--runner` option uses the retained existing binary and
verifies its hash before and after execution. It skips Cargo entirely, so a
regression cannot replace a runner used by another active check. Building a new
runner, when needed, requires a separate isolated target directory.

`check_focused.py GROUP --record PATH` runs a retained boundary fixture from
`focused/GROUP/`. It hashes the authored body, positive/control selection,
generated build text, runner and source snapshot, and records the canonical
execution root. `--runner PATH` may select an
existing isolated binary. The strict public-entry, full generic and legacy regression receipts establish
actual execution; older copied boundary receipts remain historical unless named
by the current manifest.

| Group | Paired scenarios | Boundary |
| --- | ---: | --- |
| arguments | 15 | Transactional external Value admission and original wrapper identity |
| validation | 18 | Runtime scalar outcomes versus rich admitted metadata |
| constructor | 9 | Frame metadata, integer width, source/definition/arity failures |
| argument-actions | 12 | Read-only Arg policy and preserved redirected binding |
| target-actions | 25 | All target classifications, error order, shallow mutation dispatcher |
| bridge | 3 | Frame selection, data mutation, binding publication |
| bridge-atomicity-compact | 14 | Full Frame/store failure preservation and redirected CopyTo success |
| frame-equality | 8 | Physical Frame comparison including inactive tail slots and binding kinds |
| binding-prediction | 64 | Prospective metadata versus actual copy, including MAX/error policies |
| integer-copy | 19 | Specialized integer copy versus generic copy with full-store parity |
| integer-targets | 5 | Specialized integer bridge and reference redirection |
| entry-minimal | 2 | Actual run_method Integer/Buffer payload, allocation and source kind |
| entry-minimal-strict | 4 | Independent returned Operand case and number controls |
| retirement | 2 | Actual Return and CopyObject retirement |
| decoder | 6 | Failed contribution preserves instruction cursor |
| engine | 2 | Direct preloaded stable-ID execution |
| entry | 7 | Public Value entry, chaining, capacity and failed preflight |

`check_kernels.py --const --record PATH` retains one actual constant-evaluator
positive and a changed-body negative control. The checked-interpreter suites
are separate evidence; neither is a native run. Earlier scratch bridge, retirement and frame measurements remain historical
supplemental evidence inside the exact archived bundle. The eight checked
receipts named in the current manifest and the representative const receipt
are the current canonical replay evidence.

The full generic fixture requires inline `Operand::Integer` for Integer returns
and stable `Operand::Object` for supported non-Integer returns, in addition to
checking the canonical payload, byte contents and resource effects. The strict
minimal entry fixture changes expected return case independently from expected
number, so equal numeric payloads cannot hide the wrong transport case.

Integrity check (reads retained evidence; does not rerun the compiler):

```
python3 tools/ports/acpi/interpreter/generic-execution/evidence/verify.py
```

The fixture-only integer migration wraps its existing Namespace in ObjectStore
and reads it back after execution; all original bytecode, results, error outcomes,
post-state assertions and controls are unchanged. Regression runner changes add
an explicit existing-binary option and before/after source/binary hash checks.

The read-only verifier rerenders complete generic/integer/pipeline fixtures and
both const bodies, compares exact fixture hashes, checks static focused inputs,
and matches the selected method names/expected results to every observed PASS.
The original top-level execution/pipeline receipts remain historical; their
exact recorded inputs are recovered and retained in the history archive. They
are not silently reinterpreted as current generic-runtime proof.
