# Atomic BufferField byte-write verification

Standalone module, no existing dependency or runtime edits. Inputs are
already-converted initialized bytes, canonical ObjectStore, and a direct field
ID. Fixed capacity is 256 backing/payload bytes and 64 stable object slots.

Run here:

```sh
python3 fixtures.py --check
python3 inventory.py --check
python3 reference.py --write
python3 check.py --record verification.json
python3 check_const.py
python3 verify_record.py
```

The actual checked-interpreter corpus compares every part of the entire store:
64 Values/links, all 16,384 initialized byte-array positions plus block metadata,
32 complete namespace entries/paths, and both counts. No hash/fingerprint replaces
these comparisons. The equality helper was copied into owned store_checks.py from
Cathedral's canonical owned-bytes fixtures, with all semantic Value alternatives.
Unused slots retain sentinel metadata and initialized bytes. Each control mutates
an expected byte in unused slot 63, proving the full-store comparison is active.
Success changes only the expected backing payload and block; failure expects an
identical store. Source/Owned variants, alias snapshots, partial-byte extraction,
wide writes, short/empty/long and dirty-tail payloads are covered.

Three constant positive/control pairs execute the same authored write operation
but compare its result, backing bytes/value and counters; the full-store walk is
reserved for checked interpretation because the compiler has a fixed 100,000-step
constant ceiling. Constant controls change an expected backing tail byte. This
stage is distinct from the full-store atomicity evidence.

The canonical checked runner source/lock builds in the isolated
/tmp/cathedral-acpi-buffer-field-write-runner target and must match SHA
 e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be.
Receipts bind the exact 12-file imported source/build closure, all suite Python
sources including store equality, fixtures, Rust probe/lock and runner source/lock.
Historical scratch receipts and their copied baseline remain under
history/prepublication, protected by their original manifest. No shared runner
is rebuilt from scratch Rust sources.

The actual public Object::write_buffer_field probe constructs an ordinary pinned
Interpreter with trapping host services, locks its publicly exposed real token,
performs the method call, drops the guard, and observes backing bytes. It never
creates an ObjectToken or calls unsafe gain_mut. Every row records zero forbidden
host-service calls and the one inert mutex created by Interpreter construction.
The token's host lock is distinct from the firmware GlobalLock. These are public
method observations, not actual Store opcode execution or hardware behavior.

String probes must have valid UTF-8 both before and after writes. ASCII/NUL
outcomes preserve differences from Cathedral's stricter String profile; any case
that would violate Rust String invariants is never called. Invalid destination
extents and malformed store metadata remain Omega-only (except harmless zero
width). No private method-body mirror is used.
