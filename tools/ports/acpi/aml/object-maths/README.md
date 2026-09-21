# Direct canonical arithmetic checks

The [port record](../../../../../source/libraries/acpi/aml/object-maths.PORT.md)
describes operand conversion, semantic math failures, detached division results
and the direct-ID boundary. The 506 original cases have independent Python
expectations, actual checked Omega bodies and changed expected-body controls.
Three constant pairs separately require a zero result or reject the contract.

Run from the repository root:

```sh
python3 tools/ports/acpi/aml/object-maths/inventory.py --check
python3 tools/ports/acpi/aml/object-maths/check.py
python3 tools/ports/acpi/aml/object-maths/check.py --verify-record
```

The driver uses the pinned compiler and canonical checked runner declared in
check.py and never rebuilds the shared binary. The
[runner recipe](../../interpreter/execution/README.md) documents its build.
Three independent processes use distinct temporary source/build directories and
deterministic receipt order. The receipt binds the exact 18-file used source/build
closure, fixture/driver inputs, two runner recipe files, execution root/build text
hash and binary hashes. Verification checks selected names, exact expected/observed
values, generated source hashes and all six constant proof diagnostics.

Five actual checked scratch pairs preceded publication; the canonical receipt is
a complete replay from final repository paths. No native or firmware execution,
new public Rust comparison, field/reference evaluation or opcode retirement is
claimed by these checks. Existing helper evidence remains separate.
