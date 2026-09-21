# Canonical Value conversion preflight

Scratch-only pure adapter, with no execution import, opcode or Store integration.
The source is object_conversions.omg; no shared parser, model or helper changes.
Inputs are one immutable source snapshot, its length/unit, a shared ObjectStore,
a direct source object ID, and selected IntegerSize. to_buffer additionally takes
BufferRule::ExplicitTerminated or BufferRule::PinnedObjectBytes. There is no bool
policy argument at this boundary, no output target, object allocation or mutation.

ConversionResult has semantic Failure(reason), Integer(number), or
Bytes(length,bytes[256]) cases; its default is Failure(InvalidState). This result is
not a replacement Value or arena. All Bytes are initialized and tails are zero.
Owned byte-block tails can contain initialized unrelated data; conversion copies
only the logical extent. Failure exposes no integer or byte result. Shared borrows
prevent caller state mutation, and output bytes carry no object/storage identity.

Admission stages the unsigned object_count before testing count<=64 and
object<count/object<64. It matches the direct slot Value and never follows a
Reference or NameReference at this boundary. Uninitialized, Package, Method,
service-bearing and other nonconvertible types return UnsupportedValue without
querying services. Source length/unit are relevant only when reading source bytes;
integers and Owned bytes do not consult irrelevant source metadata. A field
consults its backing value: Source-backed fields validate the snapshot while
Owned-only backing ignores unused source metadata too. All Source/
Owned owner, initialized-block, extent and ASCII checks delegate to byte_storage.

## Supported policies

| Source | to_integer | to_buffer |
| --- | --- | --- |
| Integer | Normalize to selected32/64bits | Low4/8 little-endian bytes |
| Buffer | First4/8bytes, zero extend; reject empty | Detached logical byte copy |
| String | StrictDecimalHex parser; reject empty/encoding/overflow | ExplicitTerminated adds one NUL except empty; PinnedObjectBytes excludes it |
| BufferField | Existing validated numeric reader,1..selected-width bits | UnsupportedValue |
| Other Value | UnsupportedValue | UnsupportedValue |

StrictDecimalHex is the existing named deterministic Cathedral profile: complete
ASCII decimal or0x/0X digits, no whitespace/sign/suffix, reject above widthmaximum.
It is not claimed that ACPI mandates every chosen malformed-lexical outcome.
Implicit String→Integer is deliberately absent: Table19.7 uses hexadecimal
prefix parsing without0x and an8/16digit limit. The pinned numeric parser instead
trims whitespace, accepts decimal/0x numeric prefixes, returns0 for empty inputs,
and retains raw64bit Integer/String values under FourBytes. These differences
are recorded, not hidden behind a generic implicit policy.

BufferField uses read_field_integer; direct metadata must fit the validated byte
backing and have nonzero length, and length must not exceed selected integer bits.
Backing references follow the existing reader's bounded transparent-reference
rules. Wider fields return UnsupportedValue even though the pin can read a Buffer
then convert its prefix. Full Field Unit service behavior is absent. This does not
complete the generic upstream to_integer operation.

ConversionFailure preserves Capacity, Bounds, Encoding, UnsupportedValue,
ReferenceCycle, WorkLimit, Empty and Overflow distinctions; remaining malformed
storage/helper outcomes map to InvalidState. Complete byte validation precedes
numeric parsing, so invalid late String bytes reject even after a prefix delimiter.
Length256 Strings fit PinnedObjectBytes but exceed ExplicitTerminated capacity;
empty Strings yield empty Buffers in both choices. Empty Buffer→Integer rejects.
Every operation has fixed byte capacity256, store64IDs and source1024bytes; these
are this profile's resource limits, not ACPI maxima. No generic implicit casting,
ToInteger/ToBuffer target semantics, reference resolution or opcode retirement is
included.

## Provenance and primary rules

Modified components of rust-osdev/acpi pin257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5,
[src/aml/object.rs:214/252](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/object.rs)
and [src/aml/mod.rs:2024/2057](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs).
Copyright2018 Isaac Woods, MIT OR Apache-2.0. Exact licenses are retained under
Cathedral licenses/rust-osdev/acpi; THIRD_PARTY_NOTICES.md records translation.
The inventory hashes both pinned files and keeps aggregate operations pending.

ACPI6.6 [Table19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
uses definition-block integer width and includes NUL for String→Buffer except
empty. [§19.6.138](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#tobuffer-convert-data-to-buffer)
agrees on that String rule. PinnedObjectBytes models Object::to_buffer, NOT
normative implicit conversion. [§19.6.141](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#tointeger-convert-data-to-integer)
describes explicit decimal/hex conversion and rejects empty buffers/strings.
Its first8bytes wording is reconciled here with Table19.7's selected integer width.
Raw canonical u64 Integer payloads are deliberately normalized on output; actual
public Object::to_integer retains raw values, which is a recorded profile change.
Source Buffer virtual padding and max(declared,initializer) semantics reuse the
prior canonical storage contract, including its documented correction to the pin.

## Evidence

Owned tools live under tools/ports/acpi/aml/object-conversions. The fixture set
exercises actual helper bodies and changed assertions, every successful output
byte including all256tails, Source/Owned validation, rawMAXmetadata, both width
choices, strict String parsing, direct Reference rejection and bounded fields.
Representative constant-expression controls are separate from checked execution.
reference.py calls actual public Object methods in object_reference.rs and actual
public Interpreter::load_table/evaluate separately; no private body mirrors or
ObjectToken construction are used. Raw outputs preserve policy differences and
caught upstream panics. No native execution, ABI or firmware/hardware is claimed.

Historical scratch Object-probe limitation: all12 Reference-labelled rows call
actual Object methods on RefOf(Integer42), irrespective of the corresponding
Omega fixture ReferenceKind. The207 Omega fixtures themselves use their exact
kind. These public rows attest RefOf rejection only, not all six upstream
ReferenceKind constructors. Final publication will pass exact kinds to the Rust
probe and regenerate its observations before the final real-path suite.
