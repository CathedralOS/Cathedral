# Primary Integer and Buffer String conversion

Status: **tested** at the pure bounded conversion boundary. These helpers implement
the Integer-to-String and Buffer-to-String rules from
[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules).
They do not resolve objects, select conversions, or install a target value.

`implicit_strings::from_integer(IntegerSize, u64)` produces exactly 8 or 16
hexadecimal ASCII characters. It normalizes to the chosen integer width, includes
leading zeroes, and adds no prefix. This profile chooses uppercase letters.
For example, 16 becomes `00000010` at 32 bits. It differs from the explicit
ToHexString formatting helper, which uses a prefix and variable digit count.

`implicit_strings::from_buffer(&[u8;256], length)` produces two uppercase
hexadecimal characters per byte, separated by single spaces. It has neither a
prefix nor a trailing separator. A zero-length Buffer produces an empty String.
For example, bytes 0, 10 and 255 produce `00 0A FF`. Every byte value is accepted;
this is numeric formatting, not UTF-8 or ASCII decoding of input bytes.

The semantic result is `String {length,bytes:[u8;256]}` or `Capacity`. Default
initialization produces Capacity. Length excludes the terminator; all unused
output bytes are zero, including the trailing storage after the logical String.
Only the logical input extent is read. Integer results always fit; Buffer results
require `3*n-1` characters for a nonempty input. The largest admitted Buffer has
85 bytes and produces 254 characters. A length of 86, any larger logical extent,
or maximum unsigned metadata fails before reading input or exposing output.
This is the fixed result capacity, not an ACPI maximum. There is no truncation
or partially written public result.

These are primary-rule additions alongside rust-osdev/acpi
[`src/aml/mod.rs`, pin 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs).
The pin's private `do_to_dec_hex_string` implements explicit formatting and is
not an oracle for this general conversion rule. Existing
[string-number formatting](string_numbers.PORT.md) retains that separate behavior.
The selected inventory keeps all 158 upstream anchors pending and records only
the partial Store relationship plus this module's public interface. No new
upstream method is claimed complete. Implicit operand/target dispatch and generic
String conversion remain pending.

The 334 behavior/control pairs cover 56 Integer cases and 278 Buffer cases:
both integer widths, normalization, leading zeroes, every hexadecimal digit,
all 256 single-byte values, multi-byte patterns, empty input, exact output
capacity, overflow-sized metadata and poisoned unused input. Every successful
case checks all 256 output bytes; its control changes the final expected tail
byte. Capacity controls instead demand a String result. Three constant pairs
cover 32-bit normalization of u64 MAX, the 85-byte capacity boundary and a
maximum-length failure. Host expectations use independent standard hexadecimal
formatting, while the checked runner executes the actual authored Omega bodies.
No Rust execution, native layout or hardware behavior is claimed by these vectors.

Exact used source/tool/compiler/runner hashes bind the retained receipts.
No allocator, namespace, object ID, mutable caller state or service authority is
involved. The helpers are not included in a production build root. License:
MIT OR Apache-2.0, consistent with the surrounding port; these new routines are
authored from the primary rules. Existing upstream copyright and license texts
remain in `licenses/rust-osdev/acpi` and `THIRD_PARTY_NOTICES.md`.

```sh
python3 tools/ports/acpi/interpreter/implicit-strings/check.py
python3 tools/ports/acpi/interpreter/implicit-strings/check.py --verify-record
python3 tools/ports/acpi/interpreter/implicit-strings/inventory.py --check
```
