# Direct basic-data concatenation

Status: published and tested at final repository paths. All 314 checked positive/
control pairs and three constant positive/rejecting-control pairs pass. The
receipt verifier confirms all 23 current source, fixture and runner-recipe hashes.

`object_concat::concatenate(input, length, unit, store, left, right, size)` accepts
direct canonical Integer, String or Buffer object IDs. Its failure-first
`Concatenated` result carries Failure(reason), Buffer(length, bytes[256]) or
String(length, bytes[256]). Successful byte arrays have zero unused tails. No
result object is allocated or stored and no input or ObjectStore is changed.

[ACPI 6.6 §19.6.12, Table 19.30](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#concatenate-concatenate-data)
makes the first basic operand select the second operand's implicit conversion.
The helper reuses [primary direct conversions](implicit-conversions.PORT.md) and
[existing same-type append algorithms](../interpreter/byte_concat.PORT.md).
Integer inputs produce an 8/16-byte Buffer: each normalized 32/64-bit integer is
encoded little-endian, with the left integer first. Buffer and String inputs
append the right bytes after the left bytes and preserve their result types. Logical String
results exclude the terminator. Buffer conversion of a nonempty right String
includes its terminator; an empty right String converts to an empty Buffer.

Both inputs pass complete admission before combined output capacity is checked.
The left ID/type/backing is checked first, followed by right conversion. Thus a
malformed right String cannot be hidden by an earlier output-capacity failure.
The fixed 256-byte result capacity, 64 stored IDs and 1024 source bytes are the
existing Cathedral profile, not ACPI limits. Conversion may itself fail before
append: empty byte values cannot become Integer, a nonempty 256-byte String
cannot become a terminated Buffer, and Buffer-to-String allows at most 85 bytes.
For String concatenation, all logical characters must be nonzero ASCII.
Unused Source metadata is ignored for Integer/Owned inputs, and initialized
nonlogical backing tails never appear in the result.

This is the **basic-data path only**. The same ACPI section separately names
String descriptions for other object types, including Package, Device and
Uninitialized. Those names and their full Concatenate dispatch are not supplied
here. Unsupported left/right cases return UnsupportedValue. Reference/NameReference
resolution, field/context policies, other-object formatting, target coercion,
object installation and opcode retirement remain pending. This helper must not
be described as complete Concatenate execution or use generic conversion failure
to claim the primary operator rejects all other object types.

This composition accompanies the licensed rust-osdev/acpi translation at
257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5:
[`Interpreter::do_concat`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2171).
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Exact license text and attribution
remain under licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md. The pin's mixed
String cases use decimal Integer text, lossy raw Buffer text, explicit numeric
parsing and unterminated Object Buffer conversion. These differ from the primary
conversion composition used here. No new public Rust or private-expression probe
is claimed; existing conversion/append components retain their separate evidence.

The 314 actual checked behavior/control pairs cover all nine basic input
pairings, both widths, Source/Owned backing, complete malformed storage validation,
combined capacity after conversion, unsigned normalization, empty values and
all opaque reference kinds. Every successful result checks its case, logical
length and all 256 output bytes; controls change the last expected byte or the
expected error. Representative constants exercise mixed numeric conversion,
a complete 256-byte result and failure after String formatting expands the input.
Independent Python numeric, hexadecimal and byte operations provide expectations.

The source inventory retains aggregate do_concat and resolve_as_string as pending.
No native, firmware or whole-interpreter execution is claimed.

Reproduce:

```sh
python3 tools/ports/acpi/aml/object-concat/inventory.py --check
python3 tools/ports/acpi/aml/object-concat/check.py
python3 tools/ports/acpi/aml/object-concat/check.py --verify-record
```

The driver runs three independent processes in distinct temporary directories,
retains deterministic case order and validates exact checked headers/counts.
The receipt binds the exact 18-file source/build closure, generated bodies,
fixture/driver inputs, build recipe/execution root and compiler/runner hashes.
