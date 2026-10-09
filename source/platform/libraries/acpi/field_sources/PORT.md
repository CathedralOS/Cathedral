# Primary FieldUnit source payload sequencing

Historical status: **published and tested at the recorded repository paths**. All 315 checked
positives and 315 changed-body controls passed in 277.751 seconds wall time
(831.466 seconds summed batch time, three independent processes).
Three constant positives and rejecting controls passed in 136.158 seconds.
All 38 public observations reproduce. The verifier validates 33 current input
hashes, exact upstream HEAD and all 27 pinned hashes, generated fixture/build
recipes, and exact results. This is a detached source-data algorithm, not
repeated Field execution or complete `Interpreter::do_field_write`.

The later [inline Integer entry](inline.PORT.md) shares the Integer conversion
path and owns separate current-source receipts. The measurements above retain
their original source identity rather than describing the refactored code.

The composition reuses canonical adaptations of rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0, copyright 2018
Isaac Woods. Relevant pinned bodies are `src/aml/mod.rs:2612 do_field_write` and
`src/aml/object.rs:548 copy_bits`. Original notices/licenses remain in
`THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/acpi/`. The owned inventory binds
two complete source files and keeps all 158 anchors pending, with partial targets.
Additional primary behavior is identified explicitly below, rather than being
attributed to a translated pinned method that does not implement it.

## API and validation order

`sequence::payload_at(input:&[u8;1024], input_length, unit, store:&ObjectStore,
source, size:IntegerSize, field_bits, ordinal) -> PayloadResult` accepts a direct
canonical source-object ID. Only Integer, Buffer and String slots are supported.
Reference, NameReference, BufferField and other values are rejected; callers must
perform any evaluation/resolution before this boundary. No general resolver,
canonical Value model or alternative byte-owner representation is introduced.

Validation order is explicit:

1. Zero field width fails canonical `ConversionFailure::Bounds`; width greater
   than 2048 fails `Capacity`.
2. Typed object-count/ID bounds are checked before indexing; malformed identities
   fail `InvalidState`. Unsupported direct values fail `UnsupportedValue`.
3. For Buffer/String, canonical `byte_storage::read_bytes` validates the complete
   current source or owned extent and String encoding, retaining mapped canonical
   storage errors. Source spans check input length/unit/ranges; owned values check
   owner identity, initialization and logical capacity. Unused input metadata is
   not inspected for Integer or owned storage, matching those canonical APIs.
4. Only after complete source validation does ordinal selection return a payload
   or `End {total}`. Invalid tail characters/storage still fail at a MAX ordinal.

`PayloadResult` is failure-first and defaults to `Failure {InvalidState}`.
A successful `Payload {total,ordinal,length,bytes:[u8;256]}` contains exactly
`ceil(field_bits/8)` logical bytes; every unused byte and unused high bit of the
last logical byte is zero. `End {total}` is returned for every ordinal at or
beyond the sequence count. Neither Failure nor End exposes a partial byte array.

## Primary profile and edge interpretations

[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
defines separate Integer, Buffer and String conversion rules for FieldUnits.
This profile produces one normalized Integer payload, ordered field-sized
Buffer pieces with final zero extension, or one truncated/zero-extended payload
per String character. String processing is not routed through Buffer chunking.

The following are **reasoned primary-profile interpretations**, not explicitly
quoted edge clauses in the FieldUnit rows: an empty Buffer produces one all-zero
payload under the short-buffer rule; an empty String produces no character
payloads; a String has no appended terminator pass. The latter reads the Field
row's character rule alongside the separate String-to-Buffer rule's explicit
terminator inclusion and the literal definition's non-NUL characters plus
storage terminator. These decisions are retained in fixtures and documentation.

The accepted String profile is the existing canonical non-NUL ASCII range
0x01–0x7f, with at most 256 logical bytes; non-ASCII/embedded-NUL storage fails
`Encoding`. Buffer storage also has at most 256 logical bytes. Source Buffer
virtual zero padding and an initializer larger than its declaration follow the
existing canonical byte-storage extent rule. This is not general UTF-8 support.

## Composition and bounds

Integers use canonical `normalize(size, value)` and `integer_to_buffer`; 32-bit
source values lose upper 32 bits before field truncation/extension. Their total
is always one. For Buffer length L and field width W, total is
`max(1, ceil(8*L/W))`, at most 2048. Selected part N starts at logical source bit
`N*W`, least-significant/lower bits first. Strings have total L, at most 256;
each selected character is placed in the low bits of its own field-sized vector.

All byte lengths and field widths are staged as bounded u64 scalars. The ordinal
is compared with total before multiplication; MAX ordinals therefore terminate
without overflowing. The selected multiplication additionally checks ordinal
less than 2048 and width at most 2048. Canonical `copy_bits` fills a fresh zeroed
array using the validated source length and field width. No caller-supplied
sequence Plan or cached source snapshot is accepted as proof.

There is no provider, native callback, Store installation, lock operation, live
Field read/write, or cross-call immutable-source guarantee. The caller may feed
an individual payload into the separate `field_writes::write::assemble` API,
which independently recomputes geometry and checks Previous observations. Calling
this selector repeatedly does not authorize, execute or establish the effects
of repeated hardware writes; synchronization and current Previous values remain
separate obligations under `../ADAPTER.md`.

## Pin differences and evidence

The exact pin accepts Integer/Buffer in `do_field_write`, performs one field
pass, and rejects String at entry. It does not implement the primary Buffer
repetition or String-character sequencing. No compatibility branch is provided.

The new public harness uses actual pinned `Interpreter::new`, `load_table` and
`evaluate`, original synthetic AML, and initialized Vec-backed SystemMemory
callbacks. Exact callback tuples and the entire 512-byte memory are independently
checked. Every other service traps; one inert mutex is created but never acquired.
There are no private mirrors, forged ObjectTokens or firmware/hardware fixtures.
Its 38 observations separate six Integer and ten short/equal/empty-Buffer
agreements from six wider-Buffer one-pass divergences and 16 String entry
rejections. Primary sequence payloads/counts are separate expected data, not
mislabeled public Rust results. Integer probe literals are already normalized to
their revision width; Omega separately tests oversized ordinary Integer values.

All 315 Omega cases compare actual semantic alternatives. Payload cases inspect
all 256 bytes, length, ordinal and total; controls alter expected byte 255,
including zero tails. End controls change the expected total, and failure
controls change the actual error condition. Cases cover empty/source/owned and
maximum extents; non-byte-aligned first/middle/last rounds; both Integer sizes;
high/MAX field widths and object identities; MAX byte lengths and ordinals; invalid String
tails before End; direct-reference rejection; and default Failure.
Three constant pairs cover a final nonaligned Buffer part, normalized Integer32
extended to 65 field bits, and invalid complete String storage at a MAX ordinal.
No native Omega execution or whole Field/Store integration is claimed.

Maximum checked fuel was 316,054 per body. The 315 scenarios comprise 117
Payload results, 160 End results, 37 explicit validation failures, and one
default-failure case. All retained counts and current hashes are reproducibly
validated by the owned `verify_record.py`.
