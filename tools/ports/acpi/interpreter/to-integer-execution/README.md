# ToInteger execution witnesses

The [port record](../../../../../source/libraries/acpi/interpreter/execution/to-integer.PORT.md)
defines source provenance, direct target replacement, strict numeric policy,
supported operands and the valid retirement Frame precondition.

The authored 43 actual bytecode scenarios cover both widths, Integer/decimal/
Buffer sources and Integer/String/zero-Buffer named targets, Null, Local, aliases,
nested expressions, source references, failures and full-arena scalar success.
The 25 retirement scenarios compare every Frame and ObjectStore field, including
inactive bytes, for self-conversion, argument isolation, source/destination
admission failure, error precedence, capacity and unresolved boundaries. Every
scenario has a changed expectation inside its assertion body.

```sh
python3 tools/ports/acpi/interpreter/to-integer-execution/fixtures.py
python3 tools/ports/acpi/interpreter/to-integer-execution/check.py --group execution --record tools/ports/acpi/interpreter/to-integer-execution/execution-verification.json
python3 tools/ports/acpi/interpreter/to-integer-execution/check.py --group bridge --record tools/ports/acpi/interpreter/to-integer-execution/bridge-verification.json
python3 tools/ports/acpi/interpreter/to-integer-execution/check.py --verify tools/ports/acpi/interpreter/to-integer-execution/bridge-verification.json
```

All 25 bridge pairs passed with unchanged inputs; exact receipt verification
passed. The first bytecode attempt rejected three MAX+1 control literals at
compilation. It is not counted as passing. The bridge receipt records its exact
pre-correction generator and driver; later execution evidence must preserve that
source history and distinguish unchanged generated bridge bodies from a rerun.

The runner is built from the clean Omega pin
`eaa7993a23623cd8fabf45350340479c5c9c7879` using the retained
`../execution/checked_runner.rs` and lock file. Receipts bind actual source/tool
hashes, generated bodies/build text, selected entry points, unchanged-input
checks and runner hash. Checked interpretation is distinct from constant/native
execution. No firmware or hardware access occurs.
