# Positive-extent Buffer value preparation

The later [zero-length Buffer conversion extension](zero-buffer-store.PORT.md)
admits Integer/String stores into existing empty Buffers. The profile and
source-bound receipts below record this earlier milestone, before that extension.

Same-type Buffer copying is now handled by the separate
[equal-extent named Store branch](named-buffer-store.PORT.md). This conversion
helper and its original Integer/String profile remain unchanged.

Status: tested. Final repository-path replay passed 204 checked behavior/control
pairs, three constant pairs and 54 public observations; current receipts verified.

`buffer_target_values::prepare_buffer_extent(input, length, unit, store, source,
size, extent)` prepares bytes for an explicitly supplied positive extent. It
returns the existing canonical `ImplicitResult::Buffer(length, bytes[256])` or
`ImplicitResult::Failure(ConversionFailure)`; no new nominal value/result model is
introduced. Extent must be 1..256, and the direct source slot must contain an
Integer or nonempty String. IntegerSize is the existing 32/64-bit selector.

[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
describes Integer and nonempty String conversion into an existing Buffer using
its extent, truncating or zero-extending as appropriate. The source String's
terminator is included where it fits. This API accepts the extent as ordinary
initialized geometry; the caller must establish actual target presence, extent
provenance and destination policy. No target object is read or modified.

Integer preparation reuses `integer_to_buffer` for low-order 4/8-byte encoding,
then copies at most extent bytes into initialized zero storage. The full result
length is extent, including padding beyond the selected Integer width. String
preparation fully admits canonical Source/Owned storage and ASCII first, then
copies at most extent characters into zero storage using `mid`. A following zero
provides the terminator if it fits; remaining result bytes are zero. A full
256-character String succeeds for every admitted extent without constructing a
257-byte intermediate. Every output byte beyond extent is also zero.

The validation order is ID/count, extent, source type, full String admission,
then preparation. Unsigned allocated count is staged and bounded at 64; source ID
must designate an allocated slot. Extent 0 returns Bounds and extent above 256
returns Capacity. Buffer and every other non-Integer/non-String source return
UnsupportedValue, including NameReference and all Reference cases. String backing
errors precede Empty: wrong owner, uninitialized Owned blocks, source unit/bounds,
capacity and encoding remain errors even if the logical String is empty or only
a prefix could fit. Empty is returned only after complete valid storage admission.
Integer and Owned sources ignore irrelevant source-input metadata. No failure
publishes partial bytes; default canonical ImplicitResult remains Failure(InvalidState).

Zero extent and empty String errors are **local profile exclusions**, not claims
that ACPI Store must fail for these values. Same-type Buffer source handling is
also outside this partial API. Table 19.7 lacks a Buffer-to-Buffer row, and its empty
String sentence needs an explicit destination-policy interpretation alongside
existing-extent rules. Those policies remain separately pending. This helper is
not a generic conversion dispatcher: existing `implicit_conversions::convert`
produces a new result without destination geometry. Explicit conversion operators
must not be routed through this helper merely because they have a Buffer target;
[implicit result conversion](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-result-object-conversion)
distinguishes their replacement behavior.

The licensed partial source mapping covers `src/aml/object.rs:317`
Object::replace_with_implicit_casting and `src/aml/mod.rs:2406` Interpreter::do_store,
rust-osdev/acpi pin `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0,
copyright 2018 Isaac Woods. Exact notices remain in licenses/rust-osdev/acpi and
THIRD_PARTY_NOTICES.md. Both aggregate anchors remain pending. The pinned public
replacement method always resizes a Buffer to the raw source bytes: eight bytes
for Integer, String bytes without a terminator, or a Buffer's exact contents.
It receives no IntegerSize. Public probes provide pre-normalized numeric values
when comparing the 32-bit profile and explicitly record that setup.

The 54 original public observations call that exact public method on ordinary
local owned Rust Object values. They do not instantiate an Interpreter, fabricate
ObjectToken, mirror private code, or invoke providers. Seven observations match the
prepared bytes, 25 differ in extent/padding, and 22 record excluded zero-extent,
empty-String or Buffer-source policies without claiming Omega equivalence.
Reference implementation alternatives are context, not an oracle: ACPICA's
[Buffer store implementation](https://raw.githubusercontent.com/acpica/acpica/master/source/components/executer/exstorob.c)
preserves nonzero dynamic extents but grows zero-length/static targets, while its
[String conversion](https://raw.githubusercontent.com/acpica/acpica/master/source/components/executer/exconvrt.c)
allocates length+1 even for empty Strings. No ACPICA compatibility branch is added.

The 204 original Omega cases cover both widths, eight Integer extents, seven String
lengths including 255/256, Source/Owned storage, every result byte and zero tails,
last allocated slot 63, malformed metadata/owners/counts, full encoding admission,
all exclusions and their ordering. Each changed-body control alters an expected
byte, including unused byte 255, or the expected failure case. Three separate
constant-expression pairs exercise 32-bit extension, a full 256-character String and
validated empty-String rejection. The tests use independent Python byte arithmetic.

No allocation, namespace lookup, source evaluation, reference unwrapping, target
mutation, object installation, native Omega execution or Store completion is
claimed. The [tool README](../../../../../tools/ports/acpi/aml/buffer-target-values/README.md)
contains exact reproduction commands and retained-receipt verification.
