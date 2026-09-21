# Direct logical data checks

The [port record](../../../../../source/libraries/acpi/aml/object-logic.PORT.md)
describes truth conversion, relation dispatch and the direct-ID boundary. The
346 original cases have independent Python numeric/lexical expectations, actual
checked Omega bodies and changed expected-body controls. Three constant pairs
separately require a zero result or demonstrate the failing contract.

From the repository root:

```sh
python3 tools/ports/acpi/aml/object-logic/inventory.py --check
python3 tools/ports/acpi/aml/object-logic/check.py
python3 tools/ports/acpi/aml/object-logic/check.py --verify-record
```

The driver uses the pinned compiler and canonical checked runner declared in
check.py. The [runner recipe](../../interpreter/execution/README.md) documents its
build. This driver never rebuilds the shared binary. Three independent processes
use separate temporary source/build directories and deterministic receipt order.
The receipt binds the exact 19-file used source/build closure, fixture/driver
inputs, two runner recipe files, execution root/build text and both binaries.
Verification checks every selected name, expected/observed result, generated
source hash and constant contract diagnostic.

The four-case scratch smoke preceded publication; the canonical receipt is a
complete replay from final repository paths. No native or firmware execution,
new public Rust comparison, field/reference evaluation or opcode retirement is
claimed by these checks.
