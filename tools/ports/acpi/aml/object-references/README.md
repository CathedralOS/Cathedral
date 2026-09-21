# Canonical object reference checks

The source remains in the existing `cathedral-acpi-aml` package. Tests use its
canonical `Value`, `ReferenceKind`, `Namespace`, `Object` and stable IDs.
`reference.py` executes 116 actual public immutable Rust observations at pin
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`: full/transparent unwraps, public
package-vector selection, shallow payload clone and wrapper identity. Host
pointer equality observes existing wrapped identities; no ObjectToken, unsafe
mutation, interpreter invocation, hardware access or authority is constructed.

The 160 Omega scenarios include those observations plus explicit 64-slot graph
policies: zero/oversized/MAX budgets, cycles and dangling IDs, whole-chain
package validation, capacity, atomic failures, alias/rebinding identity,
shared package elements, source/destination link preservation and inert Method
metadata copying. A negative version of every fixture changes an actual expected
result comparison. The integer executor/pipeline remain unchanged.

From the Cathedral root:

```sh
python3 tools/ports/acpi/aml/object-references/reference.py --check
python3 tools/ports/acpi/aml/object-references/fixtures.py --check
python3 tools/ports/acpi/aml/object-references/evidence.py --check
python3 tools/ports/acpi/aml/object-references/check.py --record tools/ports/acpi/aml/object-references/verification.json
python3 tools/ports/acpi/aml/object-references/check.py --const --match rust_single_0_all,zero_budget,self_cycle,package_bad_tail_after_selection,copy_method --record tools/ports/acpi/aml/object-references/const-verification.json
python3 tools/ports/acpi/aml/object-references/verify_record.py
```

Checked-interpreter execution uses clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`
and the existing checked runner source/lock from the execution harness. Its
10,000,000-step bound is an evaluator limit, separate from the API's logical
object-inspection budget. Representative const pairs use
`/tmp/cathedral-omega-eaa7993/release/omega`, SHA256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The records distinguish these stages and bind exact source/tool hashes.
No native Omega execution, ABI identity, memory access or opcode completion is
claimed. Existing parser/executor/pipeline/field checked regressions are also
refreshed after the additive canonical model change; their older const proofs
retain their original source hashes and remain historical evidence.

Dependent regressions after a canonical model change:

```sh
python3 tools/ports/acpi/pipeline/check.py --parser-regression --record tools/ports/acpi/pipeline/parser-verification.json
python3 tools/ports/acpi/interpreter/execution/check_interpreted.py --record tools/ports/acpi/interpreter/execution/verification.json
python3 tools/ports/acpi/pipeline/check.py --record tools/ports/acpi/pipeline/verification.json
python3 tools/ports/acpi/aml/fields/check_interpreted.py
python3 tools/ports/acpi/pipeline/verify_record.py
python3 tools/ports/acpi/interpreter/execution/verify_record.py
python3 tools/ports/acpi/aml/fields/verify_interpreted.py
```

These rerun the original 27, 79, 22 and 18 assertion/control pairs respectively.
Changing a proof pointer or hash is never a substitute for those runs.
