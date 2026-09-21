# Bounded numeric-string component evidence

```sh
python3 tools/ports/acpi/interpreter/string-numbers/evidence.py --check
python3 tools/ports/acpi/interpreter/string-numbers/fixtures.py --check
python3 tools/ports/acpi/interpreter/string-numbers/reference.py
python3 tools/ports/acpi/interpreter/string-numbers/check.py --record tools/ports/acpi/interpreter/string-numbers/verification.json
python3 tools/ports/acpi/interpreter/string-numbers/check_const.py
python3 tools/ports/acpi/interpreter/string-numbers/verify_record.py
```

The Rust stage calls actual pinned `Object::to_integer`. Its separately labelled
private formatter mirror contains only the pinned result construction; no
interpreter target or context behavior is executed. Reference records preserve
raw pin results even when the explicit bounded/width/ASCII profile differs.
Dependencies are locked and built offline. No upstream checkout is modified.

The Omega stage compiles actual helper and fixture bodies through the clean
pinned Omega libraries, then executes each selected body using the checked
interpreter. Every scenario has a changed-body control: parser controls change
an expected error; formatter controls invert the computed full-output comparison.
All 256 bytes are compared, so unchanged tails and complete error atomicity are
part of the behavior. `--match` permits diagnostic subsets; verification requires
the entire current case list. Source changes during a run invalidate its record.

Representative actual constant-evaluation cases independently cover u64 maximum,
32-bit overflow, maximum-value hex formatting and a multi-byte decimal buffer.
Each control must compute one and fail the checked zero-result contract. The
const and checked-interpreter stages remain separate from native Omega execution.

See `source/libraries/acpi/interpreter/string_numbers.PORT.md` for exact policies,
primary references, provenance, capacity semantics and pending generic operations.
Full ToInteger/ToDecimalString/ToHexString opcode execution is not claimed.

Current retained run: **200 checked-interpreter positives and 200 body controls pass**, plus four const-evaluation/control pairs. The checked run took 87.641 seconds; the highest per-body evaluator use was 178,290 fuel units. The host comparison passed 116 public parser calls and 76 labelled formatter mirrors, with eight explicitly inapplicable inputs. Records include source/tool/fixture hashes and preserve each distinct verification stage.

Omega pin: `eaa7993a23623cd8fabf45350340479c5c9c7879`. Compiler executable SHA-256: `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`. Checked harness SHA-256: `e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be`.
