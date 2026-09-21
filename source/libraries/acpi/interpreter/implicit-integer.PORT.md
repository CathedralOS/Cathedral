# Implicit String-to-Integer conversion

`implicit_integer::from_string(IntegerSize, &[u8;256], length) -> Conversion`
implements the String-to-Integer rule in
[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules).
The result uses semantic `Integer(value)` and `Failure(reason)` cases.
This is a primary-specification correction alongside the licensed conversion
port; it is not a transcription of the pin's decimal parser.

The logical String extent excludes its terminator. Length above 256 returns
Capacity; zero length returns Empty. The entire logical extent must consist of
nonzero ASCII bytes, consistent with Cathedral's existing String profile;
otherwise the result is Encoding. This validation includes bytes beyond the
numeric prefix and beyond the integer digit limit. Bytes outside the logical
extent are ignored. These bounds are implementation limits, not ACPI maxima.

For a valid nonempty String, conversion starts at the first byte and accumulates
hexadecimal digits, accepting both letter cases. It ends at the first non-hex
character, the logical end, or 8/16 digits for a 32/64-bit integer. It does not
skip spaces or signs, or recognize a hexadecimal prefix. Consequently `"10"`
becomes 16, `"1G2"` becomes 1, and `"0x12"` becomes zero when `x` ends conversion.
A non-hex first byte produces zero without a numeric-conversion error. Excess
digits are ignored after the width limit; this is neither an overflow error nor
truncation of the full mathematical number to its low bits.

The pin's `Object::to_integer` at
[`src/aml/object.rs:214`, revision 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/object.rs)
uses decimal or prefixed hexadecimal parsing. The existing
[string-number helpers](string_numbers.PORT.md) retain those explicitly named
policies. The explicit ASL ToInteger operator has its own decimal/hex rule in
§19.6.141; this new helper does not replace it. No new upstream anchor is marked
translated by this primary-rule addition. The wider object conversion, implicit
operand dispatch and named-target Store anchors remain pending.
The selected inventory retains all 158 aggregate anchors in the two upstream
files as pending, records the partial Store relationship, and checks the new
helper's public cases and entry point.

The retained verification contains 630 actual checked-interpreter behavior/control
pairs and three constant-evaluation pairs. Cases cover every possible first byte
at both widths, mixed case, delimiters, signs, spaces, empty strings, digit limits,
256-byte strings, invalid bytes before and after conversion stops, maximum unsigned
lengths and poisoned unused storage. Controls alter the expected success value or
replace an expected failure with a success. Constant checks include maximum 64-bit
value, 32-bit digit stopping and malformed ASCII after the numeric limit.

These are primary-rule vectors, not claimed Rust observations or native execution.
Exact source/tool/compiler/runner hashes bind the retained receipts. The helper
has no namespace, object IDs, allocation, target mutation, handler or hardware
access. Interpreter integration remains pending. Status: **tested** at this pure
bounded conversion boundary; not included in a production build root.

License: MIT OR Apache-2.0, consistent with the surrounding port. The pin's
copyright and exact notices remain in `licenses/rust-osdev/acpi` and
`THIRD_PARTY_NOTICES.md`. This new helper is authored from the primary rule and
uses the existing canonical IntegerSize; it copies no upstream implementation.

Verification:

```sh
python3 tools/ports/acpi/interpreter/implicit-integer/check.py
python3 tools/ports/acpi/interpreter/implicit-integer/check.py --verify-record
python3 tools/ports/acpi/interpreter/implicit-integer/inventory.py --check
```
