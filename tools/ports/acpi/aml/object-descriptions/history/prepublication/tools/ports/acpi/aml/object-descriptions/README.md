# Direct object type description checks

```sh
python3 tools/ports/acpi/aml/object-descriptions/fixtures.py --check
python3 tools/ports/acpi/aml/object-descriptions/reference.py --check
python3 tools/ports/acpi/aml/object-descriptions/inventory.py --check
python3 tools/ports/acpi/aml/object-descriptions/check.py --batch 10 --record tools/ports/acpi/aml/object-descriptions/verification.json
python3 tools/ports/acpi/aml/object-descriptions/check_const.py
python3 tools/ports/acpi/aml/object-descriptions/verify_record.py
```

115 actual authored Omega behavior/control pairs compare each label's semantic
case, logical length and all 256 output bytes, or the precise failure. Successful
controls change expected tail byte 255; failure controls change the expected
error. Ordinary and poisoned metadata are exercised at slots 0, 31 and 63.
Three representative constant pairs separately execute the longest label, an
invalid field payload that must remain unexamined, and a MAX object count.

The runner uses canonical checked_runner.rs against exact clean Omega HEAD
and an isolated object-description target. Every batch admits authored and
dependency bodies and executes all selected machines. Receipts bind each generated
batch, requested names, results, source/tool closure and loaded runner binary.
Constant receipts separately bind the fresh compiler and generated proof text.

`reference.py` is a static source audit of the 11 pinned literal labels, checked
against independently declared primary-table expectations in fixtures. It is
neither a public Rust call nor a private algorithm mirror. `labels.json` records
exact pin/source/tool hashes and source lines; changing its expectation alone
cannot alter the verified pin strings. All aggregate formatter/Concatenate
anchors remain pending. No target installation or runtime/hardware execution
is claimed.
