# Detached BufferField read verification

Standalone helper, no runtime or existing dependency edits. The source uses
canonical ObjectStore and Value, shared backing-byte observations, and existing
bit helpers. It returns semantic failure/integer/detached-buffer alternatives.

Run from this directory:

```sh
python3 fixtures.py --check
python3 inventory.py --check
python3 reference.py --write
python3 check.py --record verification.json
python3 check_const.py
python3 verify_record.py
```

The default checked-interpreter run checks ten cases per authored package and
executes every actual positive/control body. Controls change the expected integer,
a byte at index 255 (including unused tail), or the expected failure reason.
Fill helpers with small overrides avoid long input assignment chains. Every
successful buffer is compared across all 256 initialized bytes. Integer, a 129-bit
wide field and maximum object ID also have three constant positive/negative pairs.

The retained lock and canonical checked_runner.rs build in an isolated target,
/tmp/cathedral-acpi-buffer-field-value-runner. The runner must match canonical
SHA e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be.
The suite binds the exact 12-file imported source/build closure, scripts, fixture
catalog, public Rust probe/lock and checked-runner source/lock. Historical scratch receipts and their copied baseline remain under
history/prepublication, protected by the original manifest. No shared runner is
rebuilt from a scratch source path.

The Rust probe calls actual public Object::read_buffer_field. Its input is the
materialized logical backing corresponding to Source/Owned observations. It
records full result shape: the pin uses a 4/8-bit threshold, whereas ACPI Table
19.7 requires 32/64 bits. It also retains zero-width and out-of-range zero filling
and strings that the canonical ASCII/NUL-free profile rejects. Those differences
are reported rather than counted as equality. Full byte values agree for the
supported well-formed cases even where result type differs. Raw malformed storage
IDs/owner metadata are Omega-only fixtures; no Rust pointer or ObjectToken is
fabricated to emulate them. No private algorithm mirror is used.

The full 2048-bit example exceeds Omega’s fixed 100,000-step constant budget;
it remains covered in checked interpretation (10 million step budget). The
representative constant uses 129 bits and still checks every output/tail byte.
