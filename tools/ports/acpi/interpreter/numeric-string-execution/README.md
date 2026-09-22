# Numeric String execution evidence

The [port contract](../../../../../source/libraries/acpi/interpreter/execution/numeric-strings.PORT.md)
defines explicit decimal/hexadecimal formatting, fresh result ownership, direct
target replacement and atomic retirement. **Checked Omega execution is pending.**
Source review, host generation and public Rust comparisons are distinct stages.

The full corpus has 646 positive/control pairs. Its 160 actual AML fixtures
cover both opcodes, Integer widths, source kinds, target
replacement, retained method effects, result identities and formatting/storage
limits. The 171 direct retirement fixtures compare complete ObjectStore/Frame
state, including inactive backing and metadata. Every positive body has a
changed-expectation control. The six inherited groups retain all 315 Mid and
executor pairs: 74 Mid AML, 42 Mid retirement, 79 Integer, 55 generic, 22
pipeline and 43 ToInteger.

The checker binds every ACPI Omega input, all current generators/manifests,
borrowed fixture dependencies, generated modules, exact selections, build and
driver text and the pinned runner binary. It requires the unchanged pinned
Omega source and checks all authored/dependency bodies before interpreting
selected entries. Failed batches retain their diagnostics and remain failures.
The verifier reconstructs all generated bodies and bindings and checks every
recorded positive/control observation. `--require-complete` requires the full
declared group set and every row, rather than accepting a selected group as a
complete-corpus result.

```sh
python3 tools/ports/acpi/interpreter/numeric-string-execution/fixtures.py
python3 tools/ports/acpi/interpreter/numeric-string-execution/bridge_fixtures.py
python3 tools/ports/acpi/interpreter/numeric-string-execution/audit.py --record /tmp/numeric-string-source-audit.json
python3 tools/ports/acpi/interpreter/numeric-string-execution/check.py --batch-size 1000 --workers 1 --record /tmp/numeric-string-checked.json
python3 tools/ports/acpi/interpreter/numeric-string-execution/verify_record.py /tmp/numeric-string-checked.json --require-binaries --require-complete
python3 tools/ports/acpi/interpreter/numeric-string-execution/public/check.py --acpi-source /path/to/pinned/acpi
python3 tools/ports/acpi/interpreter/numeric-string-execution/public/check.py --verify --require-binary
python3 tools/ports/inventory.py check --checkout /path/to/pinned/acpi source/libraries/acpi/interpreter/execution/generic-inventory.json
```

`--group` narrows groups; `--match` accepts exact comma-separated names. Selected
receipts prove only those rows. The public probe uses the unchanged pinned Rust
driver with callbacks trapped, retains raw width/target/error differences and
omits canonical store edits that its public API cannot express. Those Rust
observations do not execute this Omega adapter. No native, constant-evaluator
or hardware execution is claimed.

The public receipt was independently verified: 124 actual observations, 82
result/type/named-state agreements and 42 raw nonagreements. The latter retain
14 primary-error rows without inventing Rust error equivalence, 20 value/state
differences, four evaluation errors and four load panics for initializer length
exceeding the declared Buffer size. All observations record zero forbidden
callbacks and one inert mutex. The 36 explicit omissions are 34 canonical store
edits and two Debug callback cases. Public agreement does not include Cathedral
allocation IDs or nonlogical backing bytes.
