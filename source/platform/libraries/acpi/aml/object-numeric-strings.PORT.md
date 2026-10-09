# Explicit decimal and hexadecimal String composition

Status: **tested** at the pure bounded adapter boundary: 255 final-path
behavior/control pairs, three constant pairs and 102 actual public opcode
observations passed. Source and tools are frozen; no runtime dispatch is claimed.
This is a detached, bounded result adapter, not runtime opcode dispatch.

`object_numeric_strings::convert(input, length, unit, store, object, size, format)`
accepts a direct canonical Integer, String or Buffer slot. `size` selects 32/64-bit
Integer normalization; `format` reuses `string_numbers::NumberFormat` Decimal or
Hexadecimal. The semantic result is Failure(reason) or String(length, bytes[256]).
Its default is Failure(InvalidState). Length excludes a terminator and every
nonlogical output byte is zero. There is no allocator, target installation,
namespace publication or mutable source/store borrow.

[ACPI 6.6 §§19.6.139–140](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#todecimalstring-convert-data-to-decimal-string)
defines explicit conversion of these three source types: String identity and
comma-separated per-byte Buffer values, with an empty Buffer producing an empty
String. Existing licensed `string_numbers` helpers provide numeric formatting.
Decimal values use minimal digits. Hex Integers use lowercase `0x` followed by
minimal uppercase digits; hex Buffer bytes use `0x` and two uppercase digits.
There are no spaces or trailing commas. These exact hex presentation choices
follow the pin, beyond the primary clause's unspecified padding/prefix details.
This differs from implicit Table 19.7 formatting (fixed-width Integer text and
space-separated unprefixed Buffer bytes); this module never calls implicit helpers.

Every input byte matters for Buffer formatting, including zero. Buffer output
may expand beyond the fixed 256-byte result capacity and then returns Capacity
without exposing partially formatted bytes. Decimal size depends on values;
for example 64 bytes of 255 yield 255 characters, while 65 yield 259. Hex output
requires 5*n-1 characters when nonempty: 51 bytes fit (254), 52 do not (259).
The limits are a Cathedral resource profile, not ACPI maxima.

String identity first admits the complete canonical source through `read_bytes`.
ASCII bytes excluding NUL are required throughout the logical extent. Numeric
syntax is not required for an already-String value. A validated logical copy
into fresh initialized output removes dirty nonlogical Owned tails. Source
Buffers retain canonical max(declared size, initializer length) and zero padding.
All 64-slot ID/count bounds are staged into unsigned locals before indexing.
Malformed ownership, initialization, source spans and lengths retain the existing
ConversionFailure mapping; source/storage validation precedes formatting capacity.
Unsupported top-level Reference, NameReference, BufferField and service types are
not resolved or inspected. Direct Integer conversion ignores unused source
metadata; Owned bytes likewise ignore unrelated source snapshot metadata.

The upstream component is rust-osdev/acpi
[`src/aml/mod.rs:2096`, pin 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2096),
`do_to_dec_hex_string`, copyright 2018 Isaac Woods, MIT OR Apache-2.0. Source
translation/modification notices and exact upstream licenses remain preserved in
THIRD_PARTY_NOTICES.md and licenses/rust-osdev/acpi. The pin formats raw u64
Integers; this adapter normalizes to selected width first, an explicit policy
difference. The pin accepts Rust UTF-8 Strings; this adapter retains the canonical
bounded ASCII/NUL-free String profile. Entire do_to_dec_hex_string remains pending:
argument evaluation, transparent operand references, target coercion/stores,
context contribution and operation retirement are outside this component.

Verification distinguishes actual authored Omega checked-interpreter bodies,
changed expected-body controls, representative constant proofs, and actual
public pinned Interpreter evaluations of synthetic ToDecimalString/ToHexString
AML. Public execution includes the pin's own argument/target machinery; it is
not execution of Cathedral runtime dispatch. No private Rust mirror is used.

The 255 behavior/control pairs cover both integer widths and formats, raw-width
normalization, Source/Owned values, empty through full String identity, nonnumeric
String text, decimal digit-dependent capacity, hex 51/52-byte boundaries, exact
256-character decimal output, virtual Buffer padding, initializer dominance,
complete late encoding validation, poisoned tails, malformed owners/initialization,
source extents/units, MAX metadata, opaque references and default failure. Every
successful case compares logical length and all 256 output bytes; its control
changes expected byte 255. Failure controls change the expected error case.
Three constant pairs select normalized 32-bit MAX hex, a 16-byte decimal pattern,
and a late invalid full String. Host expectations use independent standard Python
numeric formatting; checked evidence executes actual Omega helper bodies.

The 102 actual public opcode observations contain 100 String results and two
retained pin panics while constructing Buffer literals with initializers longer
than their declared size. These two cannot compare the eventual conversion;
canonical max(declared, initializer) is tested independently. Public observations
also retain raw 64-bit Integer formatting at selected 32-bit width and unbounded
Buffer text results, while Cathedral normalizes width and applies capacity.
Every call records zero forbidden host calls and one inert mutex construction.

Final receipts bind the exact 14-file source/build dependency closure, authored
tools/cases, pinned clean Omega revision, compiler binary, canonical runner source
and lock, and the runner binary loaded for every batch. Each batch records exactly
one checked-package header and one observed 0/1 pair per requested behavior.
Six constant proofs bind their generated source and compiler hash. The verifier
also validates every current public opcode row and retained scratch artifact.
The 16 historical artifacts preserve their original hashes under
`tools/ports/acpi/aml/object-numeric-strings/history/prepublication`; they are
separate historical evidence. The final manifest binds current artifacts and the
used source closure. No native execution, hardware or whole AML interpreter
completion is claimed.

```sh
python3 tools/ports/acpi/aml/object-numeric-strings/verify_record.py
python3 tools/ports/acpi/aml/object-numeric-strings/inventory.py --check
```

The tool README lists the complete behavioral, constant and public replay commands.
