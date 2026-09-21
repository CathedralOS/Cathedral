# Atomic BufferField byte writes

Status: scratch verification passed 189 full-store behavior/control pairs,
3 representative constant pairs and 89 actual public method observations. The new module
accepts already-converted raw bytes and writes a direct canonical BufferField.
It does not alter byte_storage, runtime execution, implicit conversion or Store
semantics. The source, tests and probe remain isolated until review.

`write(input, source_length, unit, store, field, payload, payload_length)` uses
an initialized source array of 1024 bytes and payload array of 256 bytes. The
mutable ObjectStore contains at most 64 stable IDs. `ByteResult` reports success
with the resolved backing ID and its unchanged logical byte length; failure
reports its canonical ByteOutcome and no published change. No IDs or namespace
entries are allocated. This is a bounded profile, not an ACPI maximum.

Admission order is fixed: validate payload length and snapshot its logical bytes;
validate the direct field ID and type; read and validate the complete backing
through `read_bytes`; then require a nonzero field wholly inside the logical
backing. A local staged backing receives `copy_bits` with truncation or zero
extension from the payload. For String backing, public `string_to_buffer` with
its unterminated helper policy validates the complete staged ASCII/NUL-free
extent into a temporary array. This is reuse of an encoding validator, not an
implicit String conversion rule. The validation output is discarded so existing
initialized nonlogical tails stay unchanged.

Only after every check succeeds are the backing ByteBlock and that object's
canonical Owned Value assigned. Both assignment values are complete before
publication; no fallible operation follows. Object sibling links, field metadata,
references, namespace entries/counters and all other objects/blocks remain
unchanged. Source-backed bytes are copied into the existing owner's fixed slot;
immutable source spans and input bytes are never made writable. Owned bytes
retain initialized tail contents outside the logical extent, matching the
existing integer-write kernel. Failure leaves the entire store exactly intact.

Top-level reference resolution is a caller responsibility. Backing Named/Local/
Arg wrappers follow the established transparent policy; RefOf/Index/Unresolved
remain opaque. The payload snapshot is independent of later backing publication,
including payloads captured from that same backing before calling this API.
Omega's exclusive/shared borrowing also rejects a raw overlapping mutable store
and shared payload borrow; the kernel does not manufacture an aliased reference.

## Provenance and primary rules

Modified translation of pinned
[Object::write_buffer_field, object.rs:291](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/object.rs)
from rust-osdev/acpi 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5. Copyright 2018
Isaac Woods, MIT OR Apache-2.0; original texts remain under
licenses/rust-osdev/acpi and attribution in THIRD_PARTY_NOTICES.md.

ACPI 6.6 [Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
defines truncation and zero extension into Buffer Fields. This adapter begins
with bytes already converted by the caller; it does not select the source's
conversion or implement Store target handling. The
[CreateField rule](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#createfield-create-arbitrary-length-buffer-field)
requires a nonempty in-range field. Strict stored-metadata rejection extends that
admission invariant to this kernel, whereas the pin lacks destination preflight.
String backing supports the internal Index-style profile; it does not broaden
CreateField's Buffer-only declaration grammar. Canonical String storage retains
its strict ASCII/NUL-free policy, unlike Rust String's UTF-8/interior-NUL profile.

## Actual public method probe

The public probe uses `Interpreter::new` with the existing trap-handler constructor,
then acquires the real token from `interpreter.object_token.lock()`. The pinned
`BaseInterpreter` exposes this field publicly (aml/mod.rs:111); ObjectToken's own
docs prescribe acquiring the interpreter-created token this way (object.rs:92).
No token is fabricated and the probe never calls `gain_mut`. It invokes actual
`Object::write_buffer_field` on a local field Object, drops the guard, then reads
the shared backing. There are no simultaneous mutable and immutable backing
references. No private algorithm is copied into the probe.

Interpreter construction creates one inert handler mutex handle and stores
numeric SystemIo register metadata. All actual host service accesses trap and
are counted; every observation must record zero forbidden calls and one created
mutex. The object-token host Spinlock is distinct from the firmware GlobalLock.
No synthetic AML Store dispatch or hardware/firmware behavior is claimed.

Public String cases must end with valid UTF-8, including ASCII and interior NUL.
The probe validates that restriction before invocation. Tests that would create
invalid Rust String storage are Omega-only; they verify atomic encoding failure.
Malformed destination extents are also Omega-only except harmless zero-width
observations. Successful bounded byte writes are compared with actual public
method results, preserving raw outcomes for deliberate profile differences.

## Evidence plan

Checked-interpreter tests compare the entire store: all 64 Object payloads and
links, all 64 full initialized byte blocks, both counters, and all 32 namespace
entries including their full paths. Unused slots contain sentinel data. Each
negative control changes an expected unused byte, proving full-store comparison
is executed. Tests cover partial-byte writes, widths through 2048 bits, empty,
short/exact/long payloads, alias snapshots, Source/Owned backing, preserved tails,
String encoding failures, invalid metadata and error ordering.

Representative constant tests execute the same write body and check its outcome,
backing bytes/value and counters; they intentionally omit the full-store walk to
fit the compiler's fixed constant budget. Whole-store atomicity is established
by checked interpretation, not claimed from the smaller constant observations.
