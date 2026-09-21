# Detached implicit data conversion

Status: tested pure helper; all 239 actual checked behavior/control pairs and
three constant-expression pairs pass from final repository paths. `implicit_conversions::convert`
selects the primary conversion for a direct canonical Integer, String or Buffer
object and requested `ConversionTarget`. Its semantic result carries an Integer,
Buffer, String or Failure; the default is Failure(InvalidState). Byte results
carry a logical length and a fully initialized 256-byte array with zero tails.
This is a detached data result, not a second canonical Value or storage arena.

The adapter implements the new-result rules from
[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules).
It composes the tested canonical byte/conversion helpers and the separate primary
implicit numeric and String formatting helpers. No existing dependency is changed.

| Source | Integer destination | Buffer destination | String destination |
| --- | --- | --- | --- |
| Integer | Normalize to selected width | 4/8 little-endian bytes | Exactly 8/16 hexadecimal characters, zero padded |
| Buffer | First 4/8 bytes, zero extended; empty rejects | Detached logical copy | Two uppercase hexadecimal digits per byte, separated by spaces |
| String | Hexadecimal prefix, at most 8/16 digits; empty rejects | Copy including NUL, except empty produces empty | Detached logical copy |

String logical extents exclude the terminator. ASCII admission covers the entire
String before conversion, even beyond the numeric prefix. Implicit numeric parsing
does not skip signs or whitespace and does not recognize `0x`; conversion stops
at the first non-hex byte without a numeric error. String-to-Buffer uses the
existing `ExplicitTerminated` helper branch because its bytes match the primary
general rule, not because an implicit operation has become an explicit opcode.

The entry stages the unsigned allocated-object count and checks it against 64,
then validates the direct ID. Only the three listed source cases are admitted.
Outer Reference and NameReference values, BufferFields, region Fields, Package,
Uninitialized and service-bearing values return UnsupportedValue. Callers choose
reference resolution and Field reads separately. Complete backing validation
delegates to `read_bytes`: source unit/extents, owned-slot identity, initialized
storage, logical length and ASCII rules retain their existing outcomes. Source
metadata is consulted only when the admitted source actually uses it; Integer
and Owned byte values ignore irrelevant source length/unit metadata.

No caller input or ObjectStore is mutated. No object is allocated, no target is
selected by name, and no method is evaluated. Capacity, Bounds, Encoding, Empty,
UnsupportedValue and InvalidState distinctions are retained; other byte-reader
failures map to their corresponding canonical conversion failure. Unexpected
internal helper results return InvalidState without exposing partial data.

This API produces a **new detached result**. It has no existing destination extent:
named Buffer targets may require additional truncation/zero-extension under the
primary table, which belongs to later Store/target handling. It therefore does
not implement implicit result storage. The fixed profile admits 64 stored IDs,
1024 source bytes and 256 result bytes. String-to-Buffer rejects a nonempty
256-byte String because the terminator cannot fit. Buffer-to-String admits at
most 85 bytes (254 output characters). Same-type copies preserve all logical
bytes and zero output tails, even when Owned backing has poisoned unused bytes.
These limits are implementation capacities, not ACPI limits.

This is a primary-policy composition alongside the licensed rust-osdev/acpi port,
pin `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`,
[`src/aml/object.rs` and `src/aml/mod.rs`](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml).
MIT OR Apache-2.0; original port copyright 2018 Isaac Woods. Exact notices remain
in `licenses/rust-osdev/acpi` and `THIRD_PARTY_NOTICES.md`. The pin conflates some
explicit/implicit parsing and uses different String formatting policies. Its
explicit Object methods are not an oracle for the corrected primary dispatch.
No new Rust observation is claimed by this component; existing component records
retain their own actual public Rust comparisons and documented differences.

The 239 behavior/control pairs cover all nine conversion combinations,
both integer widths, Source and Owned byte storage, capacity boundaries, malformed
storage and metadata, whole-String admission, exact semantic destination cases,
and poisoned unused inputs/tails. Every byte result compares all 256 bytes.
Controls alter output data or expected failure; the three representative constant
pairs exercise implicit hexadecimal parsing, integer normalization/formatting and
the Buffer-to-String capacity boundary. Host expectations use independent numeric
and byte formatting operations. Actual checked Omega body execution and constant
proofs are distinct from native or firmware execution, which are not claimed.

The inventory retains aggregate operations as pending with only a partial Store
relationship. Implicit opcode operand admission, reference/Field dispatch,
existing-target storage, object installation and runtime integration remain open.

Reproduce from the repository root:

```sh
python3 tools/ports/acpi/aml/implicit-conversions/inventory.py --check
python3 tools/ports/acpi/aml/implicit-conversions/check.py
python3 tools/ports/acpi/aml/implicit-conversions/check.py --verify-record
```

The receipt binds the exact 16-file imported source/build closure, generated
fixture bodies, driver and case manifest, runner recipe, and compiler/runner
identities. Each successful result has a paired changed expected value that must
fail; constant controls must fail their zero-result contract.
