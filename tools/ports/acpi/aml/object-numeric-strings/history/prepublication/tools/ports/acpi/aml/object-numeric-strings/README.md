# Detached explicit numeric String adapter checks

Run from Cathedral root:

```sh
python3 tools/ports/acpi/aml/object-numeric-strings/fixtures.py --check
python3 tools/ports/acpi/aml/object-numeric-strings/inventory.py --check
python3 tools/ports/acpi/aml/object-numeric-strings/reference.py --write
python3 tools/ports/acpi/aml/object-numeric-strings/check.py --batch 10 --record tools/ports/acpi/aml/object-numeric-strings/verification.json
python3 tools/ports/acpi/aml/object-numeric-strings/check_const.py
python3 tools/ports/acpi/aml/object-numeric-strings/verify_record.py
```

`check.py` builds the canonical checked-runner Rust source against exact clean
Omega HEAD eaa7993a23623cd8fabf45350340479c5c9c7879 into an isolated target. It
executes actual selected fixture machines and altered expected-body controls.
Every batch must admit its authored package and dependencies; source checking
alone is not a passing behavioral result. The used 14-file Omega source/build
closure and all authored Python tools, cases, runner source/lock and loaded
runner binary are hashed and must remain stable during execution.

`check_const.py` separately checks three actual constant expressions and their
changed expected-body controls with the retained fresh compiler. Large formatting
cases execute in the checked runner's explicitly recorded 10-million-step budget;
the constant representatives stay within the compiler's fixed 100,000-step budget.

`reference.py` uses the actual public pinned Interpreter on synthetic AML
ToDecimalString (0x97) and ToHexString (0x98) expressions, never a private mirror.
The separate canonical public probe provides inert interpreter construction and
traps all host service access. Each of 102 observations records zero forbidden
calls and one inert mutex creation. It includes 100 Strings and two recorded
panics during creation of Buffer literals whose initializer exceeds declared
length. These panics are not successful conversion evidence. Oversized formatting
results remain visible in the pin and are Capacity failures in the finite adapter.

No store mutation, generic target operation, native execution or hardware claim
is made. The current manifest binds final artifacts and current used sources;
when publication occurs, historical scratch artifacts retain their original
paths/hashes separately under history/prepublication.
