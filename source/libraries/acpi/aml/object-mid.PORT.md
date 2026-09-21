# Direct Buffer/String Mid composition

Status: published and tested at final repository paths. All 315 checked
behavior/control pairs and three constant positive/rejecting-control pairs pass.
The verifier confirms all 19 current source, fixture and runner-recipe hashes.

`object_mid::extract(input, length, unit, store, source, index, requested)` accepts
a direct canonical Buffer or String object ID. Index and requested length are
already evaluated unsigned AML Integer values; expression evaluation, Integer
width normalization and reference resolution belong to the caller. Its
failure-first `Portion` cases are Failure(reason), Buffer(length, bytes[256])
and String(length, bytes[256]). No object is allocated, installed or mutated.

[ACPI 6.6 §19.6.86](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#mid-extract-portion-of-buffer-or-string)
defines a zero-based substring preserving the source type. An index at or beyond
the source yields an empty value; an excessive requested length truncates at the
end. The existing `conversions::mid` computes available bytes by subtraction and
never forms index + requested, including for MAX unsigned parameters. Fresh
initialized output gives zero unused tails, independently of owned backing tails.

Canonical `byte_storage::read_bytes` admits the complete source before slicing.
Malformed storage or an invalid String tail therefore fails even when the request
is empty or beyond the end. Strings retain the existing non-NUL ASCII profile;
this is not a UTF-8 character indexer. Source Buffer virtual zero padding and
initializers exceeding the declared extent follow the canonical storage rules.
Owned sources ignore unused input/unit metadata. Limits remain 64 object IDs,
256 logical source/result bytes and 1024 source input bytes. These are Cathedral
profile limits, not ACPI requirements.

Direct Reference, NameReference, BufferField and other non-byte values return
UnsupportedValue; this boundary does not evaluate their payloads. Staged unsigned
object counts and IDs are checked before indexing. Storage failures preserve the
canonical ConversionFailure mapping. A wrapper's default Portion is Failure with
InvalidState; failures expose no partial output array.

The composition accompanies rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0, copyright 2018 Isaac
Woods. The source anchor is `src/aml/mod.rs:2137 do_mid`. Exact notices and licenses
remain in THIRD_PARTY_NOTICES.md and licenses/rust-osdev/acpi. The pin computes
its upper bound using index + source length, which may exceed the slice end;
the primary truncation behavior is retained instead. No new public Rust calls
or private-expression mirrors are claimed here. Existing helper observations
remain separate evidence, not execution of this composition.

The 315 behavior/control pairs cover Source/Owned Buffer and String,
empty through maximum extents, zero/exact/excessive/MAX indices and requested
lengths, complete malformed storage validation before empty selection, virtual
padding, opaque reference kinds and default Failure. Successful cases compare
the semantic case, logical length and all 256 bytes; controls alter the last
expected byte, or the expected failure case. Independent Python slicing provides
the expected portions. Three constants select a padded Buffer tail, a 128-byte
String portion and an invalid full String at a MAX index.

Full do_mid remains pending: parameter evaluation, reference policy, target
coercion, object installation and opcode retirement are not implemented here.
The owned inventory maps the partial implementation without closing aggregate
source coverage. No native, firmware or whole-interpreter execution is claimed.

```sh
python3 tools/ports/acpi/aml/object-mid/inventory.py --check
python3 tools/ports/acpi/aml/object-mid/check.py
python3 tools/ports/acpi/aml/object-mid/check.py --verify-record
```
