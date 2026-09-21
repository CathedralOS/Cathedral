# Direct named-value Store

This records the original profile. The subsequent
[equal-extent Buffer extension](named-buffer-store.PORT.md) admits Buffer sources
when both logical lengths match, including empty and self-stores. Its new
receipts are separate from this milestone's historical evidence.

Original milestone: 305 checked behavior/control pairs, three constant pairs and
297 actual public Rust observations passed at the final repository path. Those
receipts retain their original inputs. The [scalar extension](named-value-store-scalar.PORT.md)
now reruns all 305 object-source cases within 504 checked pairs and six constant
pairs, and adds destination admission plus an allocation-free Integer entry.

`named_value_store::store_value(input, length, unit, store, destination, source,
size)` operates on two already-allocated direct canonical object IDs. The caller
selects fixed named-target semantics; an ordinary ID does not establish a name
binding or this target policy. The failure-first result is `StoreResult::Failure`
with canonical `ConversionFailure`, or `StoreResult::Stored(object)`. Stored is a
destination identity receipt, not the source value of an AML Store expression.
Canonical ObjectStore, Value, storage locators and ByteBlock remain unchanged.

[ACPI 6.6 §19.3.5.5](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-result-object-conversion)
and Table 19.14 require conversion to a fixed named destination's existing type.
Explicit conversion targets and CopyObject use distinct replacement rules and
must not be routed here. The source relationship is the partial primitive-target
branch of `src/aml/mod.rs:2406` `Interpreter::do_store` and
`src/aml/object.rs:317` `Object::replace_with_implicit_casting`, rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0, copyright 2018 Isaac
Woods. Existing licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md retain notices.
Both aggregate anchors remain pending.

## Admission and conversion

The order is fixed: bounded allocated object count/direct destination ID;
destination kind; complete old byte-storage admission for String/Buffer;
canonical source preparation; then publication. Only existing Integer, String
and Buffer destinations qualify. Uninitialized, Package, all wrappers and lexical
NameReference, BufferField, methods and services are UnsupportedValue. Neither
source nor destination wrappers are resolved. No namespace entry or unrelated
object is validated; namespace entry count is irrelevant to this direct-ID API.

Old String/Buffer admission delegates to `read_bytes`, including source-span
bounds/unit, owned locator identity, initialized storage, logical capacity and
complete String encoding. Malformed old contents cannot be repaired by an
otherwise valid replacement. Integer's inactive byte block is not admitted.
These policies match the existing canonical byte-copy admission convention.

| Existing destination | Source preparation |
| --- | --- |
| Integer | `implicit_conversions::convert(..., Integer)` |
| String | `implicit_conversions::convert(..., String)` |
| Buffer | `buffer_target_values::prepare_buffer_extent(..., old_length)` |

[Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
is applied by those existing helpers, without a new conversion/byte validator or
temporary replacement arena. Integer/String destinations admit direct Integer,
String and Buffer sources with existing numeric width, empty-input, whole-String
encoding and capacity rules. String replacement may resize to 0..256 logical
characters; Buffer-to-String formatting is limited to 85 source bytes. Buffer
retains its positive admitted extent 1..256 and accepts Integer/nonempty String.
A 256-character String can be fitted without a 257-byte temporary.

The retained Buffer exclusions are local profile limits, not normative Store
errors: zero extent is Bounds, valid empty String is Empty, and same-type Buffer
source is UnsupportedValue. They also apply to self Buffer stores. Source ID
validation remains inside preparation, after destination admission. In the Buffer
helper it precedes the zero-extent check, which precedes source-kind/encoding
admission. Thus a valid zero-length Buffer and invalid source ID return InvalidState;
a valid source ID with malformed String bytes returns Bounds at that excluded
destination extent. Empty String is rejected only after its backing is admitted.
Storage/conversion failures retain canonical distinctions, including Capacity,
Bounds, Encoding, Empty and UnsupportedValue. Unexpected internal result kind or
extent returns InvalidState before mutation.

## Publication and identity

All conversion reads occur against the unchanged store. The prepared semantic
result owns initialized output bytes, so successful source/destination identity
is safe without retaining a mutable alias. Integer self-store normalizes first;
String self-store fully snapshots before replacement. Buffer self-store remains
excluded rather than silently becoming a no-op. Distinct Source descriptors may
share immutable input spans; only the destination materializes. Cross-slot Owned
locators remain invalid, not an alternative shared-ownership model.

Success preserves destination identity and Object.has_next/next, all namespace
entries/counts, all other object payloads and all other byte blocks. Integer
publication sets its canonical Integer value and resets the entire destination
ByteBlock to uninitialized, length zero and 256 zero bytes. This explicitly clears
stale inactive storage and agrees with the experimental generic integer-copy
publication convention; there is no dependency on that unpublished helper.

String/Buffer publication installs a canonical Owned locator whose owner is the
destination ID, plus a fresh initialized ByteBlock with the prepared logical
length and zero unused tail. Old Owned poison tails are reset. Source-backed
values materialize only at commit and never modify input bytes. This full-value
replacement matches canonical clone_bytes_into output normalization, whereas
BufferField updates intentionally retain untouched old backing and tails. Block
reset is Cathedral representation policy, not an ACPI hardware side effect.

There are no remaining fallible steps once final publication starts. Every failure
preserves the complete store. Existing bindings/aliases still designate the same
ID. No allocation, provider/lock/borrow authority, lookup, Field access, generic
target routing, method evaluation, frame update, opcode retirement or whole Store
completion is supplied by this component.

## Evidence

The 305 original scenarios exercise all admitted source/destination combinations,
both integer widths, Source and Owned old/source storage, String self-stores and
Integer self-normalization, excluded Buffer self-stores, shared immutable spans,
alias bindings, slot 63, malformed MAX/high-bit metadata, old malformed bytes before
source errors, and each documented Buffer exclusion/order. An independent Python
oracle uses numeric parsing/formatting and byte fitting. Every checked pair
compares all 64 semantic payloads/links, all 32 entries and 16 Path slots, all byte
block metadata, and every one of the 16,384 arena bytes. Controls alter destination
kind, identity, link, byte tail or expected failure. Default result initialization
is separately tested as Failure(InvalidState).

Three constant pairs cover Integer block reset, owned String self-store and
malformed destination before bad source. They compare all object/entry metadata
and first/last bytes per block to stay within the fixed constant evaluation
budget; exhaustive arena-byte checks remain in the checked-interpreter stage.
The retained receipt binds exact execution root, build text/hash, used production
closure, fixture/generator inputs, generated bodies, and immutable compiler/runner
identities before/after execution. No native Omega or ABI execution is claimed.

Public Rust named Store observations are a separate parent-owned probe and receipt;
they distinguish the pin's raw-byte conversion/resizing behavior from these primary
rules. This component does not equate a detached Stored receipt with the pin's AML
Store expression result. See [tool instructions](../../../../../tools/ports/acpi/aml/named-value-store/README.md).
