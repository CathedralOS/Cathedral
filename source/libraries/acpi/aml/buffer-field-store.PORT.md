# Direct BufferField source writes

Status: all 226 complete-store behavior/control pairs, three representative
constant-expression pairs and 116 public Store observations pass from final paths.

`buffer_field_store::store_value(input, length, unit, store, field, source, size)`
accepts already evaluated direct source and BufferField IDs. Integer, Buffer and
String sources are admitted and snapshotted before the existing
[atomic BufferField writer](buffer-field-writes.PORT.md) validates and updates the
field backing. The result is canonical ByteResult: success identifies the resolved
backing object and its complete byte length; failures retain ByteOutcome and
zero result metadata. No new object, error or storage representation is introduced.

[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
distinguishes BufferField copies from repeated FieldUnit writes. Integer, Buffer
and String values overwrite a BufferField with truncation or zero extension.
This API performs one data write; it does not sequence hardware FieldUnit chunks
or characters. Integer preparation reuses integer_to_buffer, including selected
32/64-bit normalization. Buffer/String sources use canonical read_bytes for complete
storage and String encoding admission. All source bytes are obtained before any
destination mutation, including when source and backing share the same object.

A String's logical bytes suffice for this bounded operation: the writer supplies
zeros beyond the logical extent, so an included terminator and further extension
have identical bits. Empty String/Buffer therefore write zeros; a 256-character
String can fill the maximum 2048-bit field without a 257-byte intermediate.
Unlike preparing an existing Buffer destination, no zero-extent String policy is
inferred: the field has a fixed, nonzero validated bit extent. This is Store's
source policy, not a generic CopyObject-to-field API.

Validation order is source count/ID, direct source type, full source admission,
then the existing field/backing checks and staged write. Source errors precede
malformed target IDs or backing. Direct Reference, NameReference, BufferField,
Package, uninitialized and service source values return UnsupportedValue; source
reference/field evaluation belongs to callers. Outer target references are not
unwrapped, while backing references follow canonical byte-storage transparency.
Integer and Owned byte sources ignore unused source-input geometry. Canonical
limits remain 64 objects, 1024 source bytes, 256-byte backing and nonzero-ASCII
String storage; malformed source tails fail even if only a small prefix is written.

The writer validates the entire backing and field range, preserves bits outside
the field, and validates the complete resulting String before its nonfallible
commit. Every failure leaves the complete ObjectStore unchanged. Success changes
only the resolved backing Value to its matching Owned case and that owner's byte
block. Existing Owned nonlogical tail bytes remain unchanged; Source backing is
materialized into initialized zero-tailed storage. Counts, links, field objects,
namespace entries, unrelated objects and all other blocks are preserved. No object
slots are allocated. Result bytes are never published before validation completes.

This is a modified component of
[`Interpreter::do_store`, pin 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2406),
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Exact notices remain in
licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md. The pin's BufferField branch
passes raw eight-byte Integers or Buffer bytes and panics for other source cases.
This helper composes the primary conversion policy with the existing checked
writer. It does not reproduce unsafe source/backing aliasing or permit invalid
UTF-8 writes into Rust String storage. The complete do_store source anchor remains
pending for target/reference policy, other destination types and context execution.

The 226 original fixtures use independent Python bit insertion and full canonical
store comparisons. They cover nonaligned and maximum fields, truncation/extension,
32/64-bit Integer sources, empty/full String and Buffer sources, Source/Owned
backing, preserved dirty tails, shared String/Buffer source and backing identities,
malformed complete source tails, invalid owners/spans/counts, opaque source and
target references, and admission/error ordering. Every checked case compares all
64 semantic object payloads/links, all 32 entries and their 16 path segments, and
all metadata plus 16,384 byte cells. Changed-body controls alter a preserved byte.
Three separate constant pairs check representative backing/value/counter results
and all 256 backing bytes; exhaustive whole-store checks stay in the checked stage.

The [public Store probe](../../../../../tools/ports/acpi/aml/buffer-field-store/reference.README.md)
retains 116 actual pinned Interpreter observations: 82 matching writes, 10 raw
32-bit Integer-width differences and 24 caught String-source panics with unchanged
backing. All service callbacks are trapped; each fresh interpreter creates one
inert mutex and makes zero forbidden calls. Four namespace aliases designate a
distinct source Buffer. Source/backing identity is exercised only in Omega: the
pinned unsafe mutable-reference paths are deliberately not invoked on overlapping
source and destination objects. No String backing is mutated by the Rust probe.
Its exact inputs, final root/build text and upstream hashes are separately verified.

No source expression evaluation, target selection, namespace binding changes, method
retirement, provider operation, lock acquisition or hardware execution occurs.
