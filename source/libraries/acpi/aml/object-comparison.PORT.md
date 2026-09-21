# Direct canonical AML comparisons

Status: all 278 actual checked behavior/control pairs and three
constant-expression pairs pass from final repository paths.

`object_comparison::compare(input, length, unit, store, left, right, size)` compares
direct canonical Integer, String or Buffer object IDs. `ObjectComparison` has
Failure(reason), Less, Equal and Greater cases; its default is Failure(InvalidState).
The comparison returns ordering only. It does not implement logical truth
operators, construct an AML Boolean object, retire an opcode or store a result.

[ACPI 6.6 §19.3.5.4](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-source-operand-conversion)
makes the first operand's type determine the required second operand type for
ordering/equality operators. The helper selects that type, validates/copies the
complete left value, then converts the complete right value using the existing
[primary implicit conversion adapter](implicit-conversions.PORT.md). Integer
comparison is unsigned after 32/64-bit normalization. String and Buffer comparison
reuse the existing lexical byte helper; Buffer length breaks an equal-prefix tie,
not an earlier differing byte. String admission remains the bounded nonzero-ASCII
profile; complete extents are validated before any comparison can short-circuit.

The first admitted left case determines conversion even when lengths differ or a
prefix alone could decide the order. Consequently an empty Buffer/String on the
right cannot convert to Integer. Conversion to String can fail for a Buffer above
85 bytes, and conversion to Buffer can fail for a nonempty 256-byte String whose
terminator does not fit. These are existing component capacities, not new ACPI
limits. Same-type empty byte values compare normally.

The allocated object count and left ID are staged as unsigned scalars and checked
before lookup. Unsupported left cases fail before reading the right ID. A malformed
left payload fails before right admission; only a fully validated left value
allows right lookup and conversion. Source bounds/units and Owned owner/extent
validation delegate to the canonical adapter. Unused source metadata is ignored
when neither admitted operand depends on source bytes. All input/store arguments
are immutable; no object slots or backing blocks are allocated or changed.

Outer Reference and NameReference objects, BufferField and region Fields, Package,
Uninitialized and service values are outside this direct data adapter. They are
not silently resolved, read or converted. Callers must implement their evaluation
and reference policies before selecting direct data IDs. The 64-object, 1024-source
and 256-result limits match the existing store profile.

This composition is part of the licensed rust-osdev/acpi port, pinned at
257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5:
[`Object::aml_cmp` and `Interpreter::do_logical_op`](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml).
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Exact notices remain in
`licenses/rust-osdev/acpi` and `THIRD_PARTY_NOTICES.md`. The pin compares Buffer
length before bytes and admits a different set of mismatched operand conversions.
This helper composes primary policies; pinned logical evaluation is not used as
its oracle. The existing byte/conversion components retain their actual public
Rust observations. No new Rust or native Omega execution is claimed here.

The 278 original cases cover every direct source-type pairing in both integer
widths, Source/Owned backing, asymmetric conversion, full unsigned values, unequal
Buffer lengths, conversion capacity boundaries, all six opaque reference kinds,
malformed IDs/counts/owners/encoding and explicit left-first error precedence.
Changed expected ordering or failure cases must reject. Three representative
constant-expression pairs are separate from actual checked body interpretation.
Expectations use independent Python numeric/byte operations, including width masks,
hexadecimal parsing/formatting and lexicographic sequence comparison.

Both aggregate source anchors remain pending: generic logical execution includes
reference/field evaluation, truth conversion, Boolean result construction and
context effects beyond this helper.

Reproduction after publication:

```sh
python3 tools/ports/acpi/aml/object-comparison/inventory.py --check
python3 tools/ports/acpi/aml/object-comparison/check.py
python3 tools/ports/acpi/aml/object-comparison/check.py --verify-record
```

The final driver uses three independent checked-runner processes, each with its
own generated source/build directory. Receipts preserve deterministic case order,
individual generated-body hashes and exact output counts, and separately record
wall time and summed batch durations. The exact 18-file used source/build closure,
fixture/driver inputs, build recipe and execution root, and runner/compiler hashes
are bound. No shared runner binary is rebuilt by this tool.
