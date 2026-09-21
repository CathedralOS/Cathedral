# Canonical Field binding boundaries

The [Bank/Index namespace milestone](../../../../../source/libraries/acpi/aml/field-indirect.PORT.md)
adds explicit Region, Bank and Index identity alternatives. These fixtures check
each alternative in both Integer widths. Direct and transparent ObjectType report
5; scalar conversion, value admission, named Store and CopyObject fail through
UnresolvedRegion without changing the complete ObjectStore. An invalid source
cannot outrank an unresolved destination.

All 32 namespace slots, 64 object slots and 16,384 byte-arena bytes are compared,
including inactive sentinel slots. Controls alter the expected final arena byte.
The namespace comparator is reused from the adjacent loader fixtures; no
production implementation is copied into the oracle.

```sh
python3 tools/ports/acpi/aml/field-bindings/fixtures.py
python3 tools/ports/acpi/aml/field-bindings/check.py
python3 tools/ports/acpi/aml/field-bindings/check.py --verify
```

The checked runner uses clean Omega pin
`eaa7993a23623cd8fabf45350340479c5c9c7879`. The receipt records source/tool/binary
hashes, generated source and package build hashes, execution root, commands,
results and elapsed time. `--verify` checks retained inputs and results only.
Source checking and checked interpretation do not claim constant evaluation,
native execution or hardware access.

All six pairs pass in the recorded isolated worktree (204.259 seconds), with
source and binary unchanged and a maximum 451,412 evaluator fuel units. Free
fixture helpers use an `fb_` prefix to avoid the pinned interpreter's known
leaf-name collision with states in imported execution modules. This changes
fixture naming only; the full comparison and production semantics are retained.

The nested comparator check extracts the actual shared bridge `fx_` helpers and
checks Region region-ID, Bank selector-ID and Index data-ID differences:

```sh
python3 tools/ports/acpi/aml/field-bindings/comparator/check.py
python3 tools/ports/acpi/aml/field-bindings/comparator/check.py --verify
```

That receipt has its own complete input snapshot and does not relabel the earlier
generic bridge corpus as freshly executed. All three pairs pass in 9.858 seconds,
with a maximum 502,409 evaluator fuel units; strict retained-input/result
verification passes.

Canonical integration with named Store, ObjectType and ToInteger separately
passed the same six pairs. The original `checked-verification.json` remains
bound to the isolated `e74cc3e` source. Verify the new canonical receipt with:

```sh
python3 tools/ports/acpi/aml/field-bindings/check.py --record tools/ports/acpi/aml/field-bindings/integrated-verification.json --verify
```
