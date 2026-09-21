# ObjectType execution witnesses

These fixtures exercise the bounded ObjectType SimpleName/Debug adapter through
the actual loader and engine, plus exhaustive direct decoder state comparisons.
The [port contract](../../../../../source/libraries/acpi/interpreter/execution/object-type.PORT.md)
records admitted grammar, scope rules, failures and primary-spec/pin differences.

There are 53 execution and 12 decoder behavior/control pairs. Controls change
expectations inside actual assertion bodies. The generator reuses the canonical
generic execution renderer and complete Frame/ObjectStore comparators. Seeded
reference descriptors and malformed metadata are explicit fixture setup, not
claims of corresponding bytecode construction support.

All 65 pairs passed on the isolated source, with 130 checked machine entries
and no evaluator filesystem attempts. Execution took 989.572 seconds; decoder
state checks took 400.466 seconds. Both retained receipts pass exact current-input
verification against 129 source hashes. These results describe the recorded
checkpoint, not later Field, graph or opcode integration.

With clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879` and the audited runner
built from `../execution/checked_runner.rs`:

```sh
python3 tools/ports/acpi/interpreter/object-type-execution/fixtures.py
python3 tools/ports/acpi/interpreter/object-type-execution/check.py --group execution --record tools/ports/acpi/interpreter/object-type-execution/execution-verification.json
python3 tools/ports/acpi/interpreter/object-type-execution/check.py --group decoder --record tools/ports/acpi/interpreter/object-type-execution/decoder-verification.json
python3 tools/ports/acpi/interpreter/object-type-execution/check.py --verify tools/ports/acpi/interpreter/object-type-execution/execution-verification.json
python3 tools/ports/acpi/interpreter/object-type-execution/check.py --verify tools/ports/acpi/interpreter/object-type-execution/decoder-verification.json
```

`--omega-source` selects the clean pinned source; `--runner` selects an existing
audited binary. The driver never replaces a binary. `--match` selects cases for
diagnosis. Receipts bind all production AML source, generator dependencies,
case lists, generated fixture/build text, selections, output and runner hash.
The verifier requires the exact current source and production root. A later
source integration makes an old receipt historical; it does not establish new
execution merely because its bytecode fixtures are unchanged.

The runner is the same audited build recorded in
[`../named-store-execution/toolchain.json`](../named-store-execution/toolchain.json):
SHA-256 `19e3f01dff2e6d7fbc9a1ee04843c4075b63a2e3c83b23f15203b6a58e9fec3a`.
That record describes its original build and smoke check; each new checked
receipt independently records the binary hash used for this component.

The separate public Rust probe reuses the 37 encoded cases without metadata
patches and records each original input. Its 27 successful value/state agreements
are supplementary comparison evidence. Other observations and 16 explicit
omissions remain in `public/verification.json`. It compiles the unchanged existing
public harness in a temporary directory against a clean pinned acpi checkout:

```sh
python3 tools/ports/acpi/interpreter/object-type-execution/public/check.py --acpi-source /path/to/acpi-at-257aa561
```

Every completed observation requires zero forbidden callbacks and one inert
constructor mutex identity. No native Omega, hardware or firmware run is claimed.
