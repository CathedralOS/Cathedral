# Implicit hexadecimal String conversion checks

Run `python3 tools/ports/acpi/interpreter/implicit-integer/check.py` from the
repository. The script executes 630 authored Omega bodies and their changed
expectation controls with the canonical checked runner, then checks three
constant pairs with Omega eaa7993. It records exact used source/tool and binary
hashes. `--verify-record` validates the retained evidence without rerunning bodies.
`--write-cases` deterministically regenerates the primary-rule vectors.

The independent host expectation uses ASCII prefix selection and Python's base-16
integer conversion, bounded to 8/16 digits. This is a specification oracle, not a
Rust public API or private mirror. It is kept distinct from actual Omega body
execution in `verification.json`. See the
[port record](../../../../../source/libraries/acpi/interpreter/implicit-integer.PORT.md)
for validation precedence, profile limits, provenance and deferred integration.
