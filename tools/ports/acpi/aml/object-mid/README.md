# Direct Buffer/String Mid evidence

See [the port scope](../../../../../source/libraries/acpi/aml/object-mid.PORT.md)
for primary slicing rules, bounded representation and deferred dispatch.

```sh
python3 tools/ports/acpi/aml/object-mid/inventory.py --check
python3 tools/ports/acpi/aml/object-mid/check.py
python3 tools/ports/acpi/aml/object-mid/check.py --verify-record
```

The retained receipt records 315 actual checked positives and 315 changed-body
controls, plus three constant positives and rejecting controls. Expectations use
independent Python byte slicing. Successful results compare the
semantic alternative, logical length and all 256 initialized output bytes.

The driver checks three separate batch processes with isolated temporary build
directories and deterministic receipt order. It binds its exact 14-file Omega
source/build closure, three fixture/driver inputs, two runner-recipe inputs,
generated bodies, execution-root build recipe, and compiler/runner binary hashes.
Verification regenerates every case and body, checks exact reported results and
validates all current input hashes. The pinned tools must be available at the
paths declared in check.py; existing receipts do not claim a portable native
binary or whole AML execution. No new public Rust calls or private mirrors are
claimed by this composition.
