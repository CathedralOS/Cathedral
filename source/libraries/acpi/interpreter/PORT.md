# AML interpreter helpers — partial ACPI-005

## Scope and status

This is a separately staged **pure helper slice**, not an AML interpreter.
The package `cathedral-acpi-interpreter` implements integer operations and bounded
initialized byte operations. It has no opcode decoder/execution loop, method
invocation, namespace writes, argument/local slots, control-flow execution,
package operations, shared object references, or operation-region access.
ACPI-005 remains incomplete. No production build imports the package.

The neighboring [AML syntax package](../aml/PORT.md) owns parsing and namespace
construction. This package currently has no dependency on it. Its scalar and
fixed-array parameters must not be mistaken for a complete `Object` translation.
Integration with its retained source spans/object arena remains pending.

## Pin and licensing

Derived from `rust-osdev/acpi` 6.1.1 at
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, specifically
`src/aml/mod.rs` and `src/aml/object.rs`. Original copyright 2018 Isaac Woods;
Omega adaptations are modified files under `MIT OR Apache-2.0`.
The [retained license texts](../../../../licenses/rust-osdev/acpi/) and
[root notice](../../../../THIRD_PARTY_NOTICES.md) apply. The parent
[license and fixture audit](../PORT.md#pin-licensing-and-exceptions) was reviewed
before translation. No firmware dump, external uACPI example, external global
lock fixture, or unpinned external test corpus is copied.

[Inventory](inventory.json) binds complete pinned bytes and every lexical anchor
in the two source files. An operation whose mathematical subexpression is
implemented but whose object/target/context effects are missing stays **pending**,
with explicit partial target mappings. Narrow helper coverage does not mark the
whole original method translated. Other pending anchors remain later work;
none are erased by an omission overlay.

## Source map

| Pinned source | Authored implementation | Preserved / changed boundary |
| --- | --- | --- |
| `IntegerSize`, `from_revision` | `integers.omg` | Revision <2 selects 32 bits; otherwise 64. Closed semantic cases replace Rust discriminants. |
| `do_binary_maths` arithmetic | `binary` | Add/subtract/multiply, quotient and remainder, shifts, AND/NAND/OR/NOR/XOR; object conversion and target stores pending. |
| `do_unary_maths`, `do_logical_op` | `find_set_left/right`, `bitwise_not`, `logical_not`, `logical` | Integer-only operations; mixed types and buffer/string comparisons pending. |
| `do_from_bcd`, `do_to_bcd` | `bcd.omg` | Checked pure BCD conversions; context/op retirement pending. |
| `copy_bits` | `buffer_fields.omg::copy_bits` | Arbitrary bit offsets, partial-byte preservation, source zero extension. Preflight destination range; fixed capacity and disjoint borrows. |
| `read_buffer_field`, `to_integer` | `field_to_integer`, `buffer_to_integer` | Pure bit extraction and little-endian integer result; no shared object resolution. String-to-integer conversion pending. |
| `do_to_buffer`, `Object::to_buffer` | `integer_to_buffer`, `string_to_buffer` | Four/eight integer bytes. Explicit string conversion includes terminator except empty string; implicit conversion excludes it. Buffer identity is caller-owned, not an object clone API. |
| `do_to_string` | `buffer_to_string` | Bounded ASCII bytes before first NUL or maximum; logical length excludes NUL. |
| `do_mid` | `mid` | Initialized byte window with clamped output length; no string/object dispatch or target store. |

The five crate-authored `object.rs` unit-test scenarios are translated to actual
Omega calls. Selected buffer/ToBuffer expressions from `to_integer.asl` and
`to_x.asl`, the value chain from `incdec.asl`, and the returned expression from
`logical_not.asl` are likewise translated. They do **not** execute the surrounding
ASL method/namespace harness. [Case metadata](../../../../tools/ports/acpi/interpreter/cases.json)
records that distinction individually. Unsupported string parsing and remaining
ASL scenarios are not reported as passing.

## Explicit profile and deviations

- Fixed storage is 256 initialized bytes per input/output. A length over 256
  returns `ERROR_CAPACITY`; array tails outside the returned logical length are
  untouched. Copies require separate source and mutable destination borrows.
  Destination extent failure is detected before mutation. Source missing bits
  become zero, including an arbitrarily large source bit offset, without adding
  overflowing offsets. This preserves the pin's zero-extension rule while
  replacing possible destination indexing panics with a result.
- Integer operands/results normalize to the selected 32/64-bit width. Logical
  true is all ones at that width. The pin performs many operations in `u64`
  regardless of revision. Wrapping arithmetic is explicit at each node;
  division/modulo by zero return error rather than panic.
- `Not` is bitwise complement. The pin implements zero/nonzero logical negation
  there. `FindSetLeftBit` returns a one-based highest-set position; the pin uses
  leading-zero count plus one. Lowest-set search is also width-normalized.
- Shifts with a normalized count at least the integer width return zero. This is
  the finite-width zero-fill interpretation of the primary shift operation,
  rather than Rust's modulo-width wrapping shift behavior in the pin.
- BCD rejects any nibble above nine and decimal values needing more than 8/16
  BCD digits. The pin does not check these failures. Error handling here is a
  deterministic Cathedral profile for malformed operands, not an assertion that
  ACPI specifies these numeric error codes.
- `EmptyPolicy::Reject` implements the primary explicit ToInteger empty-buffer
  prohibition. `PinnedZero` explicitly preserves the pin's empty-buffer zero.
  Neither option implies a complete generic ToInteger implementation.
- Strings here are ASCII byte extents excluding a terminator. NUL or non-ASCII
  within the supplied String extent rejects. Buffer-to-string stops before NUL
  and rejects non-ASCII before the stop, leaving output unchanged on error.
  The pin uses Rust UTF-8 and includes a found NUL in `do_to_string` output.
- `mid` clamps to `length - index`, then to requested length. An out-of-range
  index produces empty output. The pin computes `min(index+length,
  index+source_length)`, which can overrun or overflow. No addition involving
  the untrusted requested length is needed here.
- Field extraction returns the low selected-width bits, even when the field is
  longer. Full read-buffer-field Object result selection remains pending; the
  pin compares field bits against the integer byte count in that path.

Primary references: [ACPI 6.6 data types and conversion rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#asl-data-types),
[FindSetLeftBit](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#findsetleftbit-find-leftmost-set-bit),
[Not](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#not-integer-bitwise-not),
[ASL operator reference](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html)
sections 19.6.123–124 and 19.6.137–143. These are behavior references, not an
Omega layout/ABI claim. Native representations have no selected foreign layout.

Errors are inert local tags: 0 success, 1 zero divisor, 2 invalid BCD,
3 overflow, 4 capacity, 5 destination bounds, 6 forbidden empty buffer,
7 invalid ASCII String. `IntegerResult.remainder` is the actual remainder for
Divide and an internal scan carrier elsewhere; callers consume only the fields
specified by their operation. No physical or firmware authority follows.

## Verification

**Semantic helper stage tested, 2026-09-20:** all 150 scenarios and three
body-mutating controls pass with the compiler below. The complete corpus
source-checks as 14 source files. Execution was the original 141-case batch
plus six capacity and three final-byte additions; the canonical runner now
reproduces their union. This does not advance the whole interpreter stage.

The canonical runner checks all authored fixtures, then demands semantic
constant evaluation of the real Omega helper bodies in separate two-case
fixtures. Body-mutating negative controls change arithmetic, copied bytes and
invalid-BCD expectations; each must fail with `TEST_RESULT == 1`, so merely
accepting a source file or a default zero cannot pass the suite.

Compiler: exact clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, isolated
`/tmp/cathedral-omega-eaa7993/release/omega`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The runner prints the actual binary hash for every execution. Native artifact,
AML bytecode execution, external interpreter comparison, firmware/hardware and
production integration are **not run**. See the tools README for current results
and commands; do not infer those later stages from semantic checking.

## Remaining interpreter and service boundary

Complete ACPI-005 still needs values and conversions, object/reference identity,
methods/arguments/locals, control flow, packages, fields and target stores,
namespace binding, comprehensive upstream semantic scenarios, and explicit
limits across execution. Current loops are bounded but do not establish the
whole ACPI-006 resource-limit milestone. Large cases may also exceed the
compiler evaluator's own work budget; that is separate from a runtime profile.

An OperationRegion declaration remains inert AML metadata in the syntax
package. This helper package exposes no read/write callback, ambient handler,
physical mapping, I/O primitive, mutex/event/timer service, or successful stub.
A future evaluator must stop on an unresolved external operation and return an
explicit unsupported/unresolved service outcome; implementing that evaluator
boundary is pending. The [adapter specification](../ADAPTER.md) separately
requires named grants, bounds, lifetimes and cleanup before live access.
