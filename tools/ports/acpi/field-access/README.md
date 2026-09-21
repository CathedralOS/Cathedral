# Normal linear Field arithmetic evidence

Published bounded module and original authored fixtures. No firmware fixture,
hardware invocation, native Omega publication or completed Field evaluator claim.
Production semantics and partial source mapping are in
`source/libraries/acpi/field_access/PORT.md`.

Run from the repository root:

```sh
python3 tools/ports/acpi/field-access/fixtures.py
python3 tools/ports/acpi/field-access/reference.py --write
python3 tools/ports/acpi/field-access/compare.py --write
python3 tools/ports/acpi/field-access/check.py
python3 tools/ports/acpi/field-access/check_const.py
python3 tools/ports/acpi/field-access/verify_record.py
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/field_access/inventory.json
```

`reference.py` uses the exact pinned Rust crate and isolated Cargo target
`/tmp/cathedral-field-access-public`. Its actual public callback observations
are compared with independent integer-interval arithmetic by `compare.py`.
`reference-verification.json` binds its source, lockfile and complete pinned
Rust source hashes. The 362 observations include deliberate strict-policy
corrections, retained separately from ordinary agreements.

`check.py` invokes the existing immutable checked interpreter runner (override
with `--runner`), admits actual source bodies and executes 451 positive/control
pairs in bounded batches. `--case NAME` selects a diagnostic subset, whose record
is explicitly marked selected. `check_const.py` uses the pinned compiler (override
with `--compiler`) for representative compile-time proof pairs. No tool rebuilds
the shared runner. All executed stages bind the used production dependency
closure and harness inputs before and after execution; record validation checks
exact generated source hashes and results as well as current input hashes.

The initialized Plan has 257 chunks. Positive comparisons inspect all 257 slots,
not merely summary fields or logical count. Body controls change the expected
slot-256 mask (zero tail or final live chunk) for plans and an expected numeric/error condition for scalar
results. Error alternatives carry no partial Plan. Tests separately exercise
u64 MAX/high-bit offsets and lengths, invalid metadata consistency, all three
update rules, read shape at 32/64 bits and the 2048-bit capacity boundary.

Final repository verification: **451 checked positive/control pairs**, **three
constant positive/rejecting-control pairs**, and **362 actual public observations
with independent arithmetic/access comparisons** all pass. Current records bind
26 explicit inputs. Whole-method inventory remains partial (two files, 158
pending anchors); this count deliberately does not label full Field execution
translated. No native or provider execution claim follows.
