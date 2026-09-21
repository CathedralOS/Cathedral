# Same-type concatenate validation

From Cathedral:

```
python3 tools/ports/acpi/interpreter/byte-concat/fixtures.py --check
python3 tools/ports/acpi/interpreter/byte-concat/reference.py
python3 tools/ports/acpi/interpreter/byte-concat/evidence.py --check
python3 tools/ports/acpi/interpreter/byte-concat/check.py --record tools/ports/acpi/interpreter/byte-concat/checked-verification.json
python3 tools/ports/acpi/interpreter/byte-concat/check_const.py
python3 tools/ports/acpi/interpreter/byte-concat/verify_record.py --write
python3 tools/ports/acpi/interpreter/byte-concat/verify_record.py
```

`check.py --match TEXT` selects named scenarios for diagnosis. The canonical full
run checks every body and control through clean pinned Omega eaa7993, using the
shared read-only execution runner and lock. `check_const.py` additionally checks
integer width truncation, full buffer capacity and string error precedence with
actual constant evaluation. All 256 output bytes are compared for every scenario;
controls flip expected byte 255 inside the test body, including preserved tails.
The five-file Cathedral dependency closure and execution tools are hashed before
and after runs. Source and fixtures stay frozen during validation.

`reference.py` retains exact pinned file/license hashes and Cargo lock. It calls
actual public Object conversion/access APIs; its private `do_concat` append
expressions are explicitly labelled mirrors. It does not execute that private
method, a generic AML operation, target storage or live handlers. Rust permits
some NUL strings and unbounded result lengths rejected by the bounded Omega
profile; invalid UTF-8 and unrepresentable logical extents are explicitly omitted.
`reference.rs` is generated and retained for review.

The complete source inventory keeps whole `do_concat` pending. Generic
conversion/type-name formatting/Store/opcode integration is separate work.
No existing interpreter helper or build file is modified by this component.
