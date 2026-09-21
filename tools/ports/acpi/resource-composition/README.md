# Bounded resource-template concatenation evidence

The production component is
[`resource_composition/PORT.md`](../../../../source/libraries/acpi/resource_composition/PORT.md).
The 25 original scenarios run the real Omega composition, resource validation
and copy bodies. Every checked-execution scenario checks all 4096 output bytes,
including untouched tails and complete output preservation after errors. Each
paired control changes the output comparison and must return one instead of zero.

`verification.json` records 25 positive selections and 25 controls, all passing
with clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`. This is checked
interpreter execution, without native publication, hardware or AML dispatch.
The source closure includes all resource parser modules and the transitive
`fixed_bytes`, `bytes` and `headers` dependencies, plus their build root.

`check_const.py` separately selects empty inputs, two resource descriptors,
one-byte right input and insufficient capacity. Constant evaluation checks the
complete small result, its eight-byte prefix including unused bytes, and a
distant tail sentinel at offset 4095. Full-buffer preservation is established
by the checked suite; it is not inferred from the focused constant checks.

`reference.py` extracts the exact private pinned `ConcatRes` result block and
changes only the final wrapped Object construction to return its byte vector.
There are 23 such Rust observations, nine successful-profile byte agreements
and two impossible host logical extents omitted. This is a labelled private
body mirror, not a call to the actual interpreter or its Store operation.
Malformed buffers accepted by the pin remain explicit differences from the
strict Cathedral resource profile.

From the Cathedral root:

```sh
python3 tools/ports/acpi/resource-composition/fixtures.py --check
python3 tools/ports/acpi/resource-composition/reference.py
python3 tools/ports/acpi/resource-composition/check.py --record tools/ports/acpi/resource-composition/verification.json
python3 tools/ports/acpi/resource-composition/check_const.py
python3 tools/ports/acpi/resource-composition/evidence.py --check
python3 tools/ports/acpi/resource-composition/verify_record.py
```

The checked runner uses the pinned clean sibling Omega checkout and Cargo's
retained offline lockfile. The constant check uses the isolated pinned compiler
at `/tmp/cathedral-omega-eaa7993/release/omega`. Keep source and verification
tools unchanged while either stage runs. JSON receipts bind tested inputs by
SHA-256; output counts alone are not current-source evidence.
The inventory reproducer and receipt verifier were added after the checked run;
its original receipt is preserved. The verifier requires every current production
dependency to appear in both execution-stage records and checks all recorded
input hashes, rather than extending historical receipts with untested inputs.
