# AML loader-to-executor tests

These are original synthetic AML TermLists, not copied firmware or externally
licensed sample binaries. See the package's `PORT.md` for provenance and limits.

From the repository root:

```sh
python3 tools/ports/acpi/pipeline/fixtures.py --check
python3 tools/ports/acpi/pipeline/check.py --record tools/ports/acpi/pipeline/verification.json
python3 tools/ports/acpi/pipeline/check.py --parser-regression --record tools/ports/acpi/pipeline/parser-verification.json
python3 tools/ports/acpi/pipeline/verify_record.py
python3 tools/ports/acpi/interpreter/execution/check_interpreted.py --record tools/ports/acpi/interpreter/execution/verification.json
python3 tools/ports/acpi/interpreter/execution/verify_record.py
python3 tools/ports/acpi/interpreter/execution/check_frame_const.py
python3 tools/ports/acpi/aml/check.py --case load-method --case load-rollback --jobs 2
```

The checked harness is the existing Cathedral Rust consumer of the clean pinned
Omega checkout. It uses offline package preparation and checking for inspection,
then `interpret_entry` on each selected actual Omega test body. No Omega source
is modified. Source checking can take several minutes in the pinned compiler's
aggregate alias analysis before interpretation starts.

The pipeline suite covers declaration capture, nested calls, cross-scope aliases,
original-name rebinding, method redeclaration, an owned source copy, persistent
namespace effects, serialized/source/payload rejection, maximum unsigned inputs,
failed-load rollback, object slot 63, capacity rollback, occupied sidecar rejection
and the explicit low-level incremental-source boundary. Each expected result has
a changed-body control. `--match` selects a subset for diagnosis; only complete
records pass `verify_record.py`.

The parser regression runs the original 27 assertion bodies and exact mutations
through the checked interpreter. This stage does not replace historical const
proofs with new hashes. The separate representative parser command still uses
constant evaluation and requires rejection of a changed body. The frame command
similarly retains a direct const proof of real executor observation checks.

The companion JSON records identify the exact checked-interpreter stage and
source bytes. Native execution, hardware access, ABI conformance and full AML
semantics are not established by these tests.

The retained current run passed 22 pipeline pairs, 27 unchanged parser pairs,
79 existing executor pairs and 18 unchanged field-parser pairs. Each pair is a
positive body and its changed expected-value control. The pipeline run took
380.560 seconds and used at most 75,688 evaluator fuel units per selected body.
The parser run took 189.025 seconds; the executor regression took 280.577 seconds.
The current method/rollback parser const pairs and method-frame const pair also
passed; `const-verification.json` retains their separate stage and source hashes.

The field dependency record is `../aml/fields/checked-verification.json`; run
`python3 tools/ports/acpi/aml/fields/verify_interpreted.py` to check it. Historical
parser/field const records retain their original source hashes and point to the
current regressions separately. The compiler executable hash is
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`; the checked
harness hash is `e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be`.
