# AML interpreter helpers — partial ACPI-005

## Scope and status

This package is the separately staged **pure helper slice**. The child
[integer method executor](execution/PORT.md) now supplies bounded bytecode
execution using these helpers and the static AML namespace.
The package `cathedral-acpi-interpreter` implements integer operations and bounded
initialized byte operations. It has no opcode decoder/execution loop, method
invocation, namespace writes, argument/local slots, control-flow execution,
package operations, shared object references, or operation-region access.
Those statements describe this helper package; the child executor adds integer
methods, targets and control flow. ACPI-005 remains incomplete. No production
build imports either package.

The neighboring [AML syntax package](../aml/PORT.md) owns parsing and namespace
construction. This package currently has no dependency on it. Its scalar and
fixed-array parameters must not be mistaken for a complete `Object` translation.
The child executor integrates its retained source spans and object arena with
method-definition observations. The [single-source pipeline](../pipeline/PORT.md)
now captures these at declaration and owns their immutable source snapshot.

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
| `read_buffer_field`, `to_integer` | `field_to_integer`, `buffer_to_integer` | Pure bit extraction and little-endian integer result; no shared object resolution. String parsing is supplied by the separate [string/number helpers](string_numbers.PORT.md); generic Object dispatch remains pending. |
| `do_to_buffer`, `Object::to_buffer` | `integer_to_buffer`, `string_to_buffer` | Four/eight integer bytes. The primary String conversion includes the terminator except for an empty String. The helper's `explicit=false` branch reproduces pinned `Object::to_buffer` bytes without the terminator; it is a compatibility choice, not the primary implicit-conversion rule. Buffer identity is caller-owned, not an object clone API. |
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
- [ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
  also includes the String terminator in String-to-Buffer conversion. The helper's
  `explicit=false` spelling describes its pinned Object-method branch; callers
  must not use that flag as a general implicit-versus-explicit ACPI policy switch.
  Implicit String-to-Integer uses hexadecimal digits without a `0x` prefix;
  `StrictDecimalHex` implements the separate explicit ToInteger profile.
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

The child executor passes 79 checked-interpreter cases and 79 changed-body
controls for integer methods, arguments/locals, existing named targets, aliases
and bounded control flow. Its current frame-admission constant proof also passes
a changed-body control; the complete final bytecode suite uses the distinct
checked-interpreter stage. See its PORT for exact hashes and evidence.

Complete ACPI-005 still needs generic values/conversions and references, packages,
fields, dynamic namespace binding, multi-unit source
management and comprehensive upstream semantic scenarios. Current loops are bounded but do not establish the
whole ACPI-006 resource-limit milestone. Large cases may also exceed the
compiler evaluator's own work budget; that is separate from a runtime profile.

The separate [resource-template composition](../resource_composition/PORT.md)
component passes 25 checked-interpreter pairs and four focused const pairs. It
validates both detached templates before writing, removes their EndTags and
appends one new EndTag. Full 4096-byte output preservation is checked, with
23 explicitly labelled private Rust result-block mirrors. Generic ConcatRes
argument/target dispatch remains pending.

An OperationRegion declaration remains inert AML metadata in the syntax
package. This helper package exposes no read/write callback, ambient handler,
physical mapping, I/O primitive, mutex/event/timer service, or successful stub.
The child evaluator returns explicit unresolved region, synchronization and
service outcomes; it installs no live handlers. Broader object/service dispatch
remains pending. The [adapter specification](../ADAPTER.md) separately
requires named grants, bounds, lifetimes and cleanup before live access.

The [string/number component](string_numbers.PORT.md) adds strict and pinned-ASCII
String-toInteger parsing, width-aware integer formatting and bounded buffer
decimal/hex formatting. All 200 checked-interpreter cases and 200 controls pass,
plus four constant-evaluation pairs; reference evidence includes 116 actual
public Rust calls and 76 labelled private formatting mirrors. Generic opcode
argument/target/context integration remains pending.

The [same-type concatenation component](byte_concat.PORT.md) adds bounded
integer-pair, Buffer and ASCII String concatenation with complete preflight and
preserved output tails. All 63 checked-interpreter pairs and three const pairs
pass. Its 49 private append-expression observations are distinguished from 162
actual public Rust conversion/access calls. Operands are already converted;
generic type conversion, result storage and opcode integration remain pending.

The [byte comparison component](byte_comparison.PORT.md) adds bounded ASCII
String ordering and explicit primary lexicographic versus pinned length-first
Buffer ordering. All 81 checked-interpreter pairs and four const pairs pass;
62 public Rust comparisons retain the documented profile differences. The
[direct object comparison adapter](../aml/object-comparison.PORT.md) now adds
primary right-hand conversion with 278 checked pairs and three constant pairs.
Reference/field evaluation and complete generic logical opcodes remain pending.

The [implicit String-to-Integer helper](implicit-integer.PORT.md) implements the
primary hexadecimal prefix rule with 8/16-digit stopping. Its 630 checked pairs
and three constant pairs cover complete ASCII admission and width limits. It is
separate from the existing explicit decimal/hex parser; generic implicit operand
and named-target conversion dispatch remain pending.

[Primary String formatting](implicit-strings.PORT.md) adds fixed-width Integer
hexadecimal text and space-separated Buffer byte pairs. All 334 checked pairs
and three const pairs verify complete initialized outputs and capacity rejection.
These helpers complement the explicit formatting policies. The separate
[implicit conversion adapter](../aml/implicit-conversions.PORT.md) selects all
nine direct Integer/String/Buffer conversions with 239 checked pairs and three
constant pairs; reference/Field resolution and target mutation remain pending.

The [canonical object conversion adapter](../aml/object-conversions.PORT.md)
connects the existing byte/numeric helpers to direct ObjectStore IDs, with 207
checked pairs, three const pairs and 166 actual public Rust observations. It
returns detached Integer/byte results after complete validation; execution and
target writes remain pending.

[Detached normal Field read assembly](../field_values/PORT.md) adds complete
Integer/Buffer result construction from validated numeric words, with 301 checked
pairs, three const pairs and 259 actual public Rust read observations. It preserves
lock requirements without granting access. Live reads, field dispatch, Bank/Index
and object installation remain pending.

[Detached BufferField reads](../aml/buffer-field-values.PORT.md) compose the bit
helpers with canonical byte storage and full Integer/Buffer result selection.
The 281 checked pairs, three const pairs and 136 actual public Object observations
cover fields through 2048 bits and strict bounds/encoding. The bounded upstream
read anchor is translated; field opcode dispatch remains pending.

[Detached normal Field write assembly](../field_writes/PORT.md) turns already-converted,
exactly field-sized bits into complete native-width numeric write records. All
454 checked pairs, three constant pairs and 126 actual public Rust probes pass.
Geometry, payload/count checks, Preserve input requirements and unmet lock metadata
are retained. Source conversion/repeated writes, provider access, Bank/Index and
Store/evaluator integration remain pending; aggregate source counts are unchanged.

[Field source sequencing](../field_sources/PORT.md) prepares one normalized Integer payload,
ordered Buffer pieces, or individual String characters for a normal Field.
All 315 checked pairs, three constant pairs and 38 public Rust observations pass;
the observations retain the pin's single-pass and String-rejection differences.
Empty-source and terminator choices are documented primary-profile interpretations.
Repeated execution, provider/lock access and Store integration remain pending;
aggregate source counts are unchanged.

[Direct basic-data Concatenate](../aml/object-concat.PORT.md) composes all Integer/String/Buffer
pairings with primary right-hand conversion, little-endian integer encoding and
complete admission before combined capacity checks. All 314 checked pairs and
three constant pairs pass. Other-object descriptions, reference/field policies,
object installation and opcode retirement remain pending; aggregate counts are
unchanged.

[Atomic BufferField byte writes](../aml/buffer-field-writes.PORT.md) stage already-converted payloads,
validate the full field and backing, and publish only after String encoding checks.
All 189 whole-store behavior/control pairs, three representative constant pairs
and 89 actual public Object method observations pass. One bounded source anchor
closes; source conversion, target handling and Store execution remain pending.

[Direct Buffer/String Mid](../aml/object-mid.PORT.md) validates complete canonical backing,
preserves the source type and slices without overflowing index + requested length.
All 315 checked pairs and three constant pairs pass, including malformed tails
before empty selection and full zero output tails. Parameter evaluation,
reference policy, target writes and opcode retirement remain pending; aggregate
source counts are unchanged.

[Direct Buffer ToString](../aml/object-to-string.PORT.md) admits complete backing before selecting the
ASCII prefix ending at NUL or the requested maximum. All 195 checked pairs and
three constant pairs pass. The 44 public Rust observations retain 33 agreements
and 11 documented NUL/UTF-8 differences. Reference evaluation, target writes and
opcode execution remain pending; aggregate source counts are unchanged.

[Explicit numeric String composition](../aml/object-numeric-strings.PORT.md) handles direct Integer, String
and Buffer sources for decimal/hexadecimal formatting. All 255 checked pairs,
three constant pairs and 102 actual public opcode observations pass; the public
record retains 100 String results and two pinned literal-construction panics.
Explicit formatting and width/capacity differences are documented. Operand
resolution, target writes and opcode retirement remain pending; counts are unchanged.

[Direct String-name lookup](../aml/string-lookup.PORT.md) joins canonical String admission, textual
ASL name parsing and scoped namespace search, returning an object ID and path.
All 97 checked pairs and three constant pairs pass, including scope validation,
error precedence and all initialized path segments. Target evaluation, reference
policy and full DerefOf execution remain pending; aggregate counts are unchanged.

[Positive-extent Buffer preparation](../aml/buffer-target-values.PORT.md) prepares Integer/nonempty String bytes for a caller-supplied
Buffer extent, preserving that extent through truncation and zero padding. All
204 checked pairs, three constant pairs and 54 public replacement observations
pass; the public record distinguishes seven agreements, 25 differences and 22
excluded-policy observations. Zero extent, empty String and Buffer sources remain
outside this profile. Target provenance, mutation and Store execution are pending;
aggregate source counts are unchanged.

[Direct logical results](../aml/object-logic.PORT.md) compose primary Integer truth conversion and
left-directed relational comparison over canonical basic values. All 346 checked
pairs and three constant pairs pass, including exact 32/64-bit Boolean results,
full right admission and malformed-tail errors. Operand/reference evaluation and
context retirement remain pending; aggregate source counts are unchanged.

[Direct object descriptions](../aml/object-descriptions.PORT.md) supply the eleven represented nonbasic
Concatenate labels without reading or validating object payloads. All 115 checked
pairs, three constant pairs and eleven static pinned-label audits pass; receipts
bind the final execution root and generated build text. General Concatenate
dispatch and opcode execution remain pending; aggregate counts are unchanged.

[Direct arithmetic results](../aml/object-maths.PORT.md) admit canonical Integer/String/Buffer
operands before width-normalized mathematics. All 506 checked pairs and three
constant pairs pass, including separate quotient/remainder results and semantic
conversion, divide-by-zero and BCD failures. Target writes, operand/reference
evaluation and opcode retirement remain pending; aggregate counts are unchanged.

[Package Index construction](../aml/package-index.PORT.md) validates the advertised member chain and
allocates one fresh RefOf wrapper preserving the selected element's identity.
All 65 complete-store checked pairs, three bounded constant pairs and 19 public
Index observations pass. Source evaluation, target Store and opcode retirement
remain pending; aggregate counts are unchanged.

[Direct BufferField source writes](../aml/buffer-field-store.PORT.md) compose Integer/Buffer/String
admission with atomic backing updates. All 226 complete-store checked pairs,
three representative constant pairs and 116 public Store observations pass.
The public record retains ten width differences and 24 String-source panics;
shared source/backing identity is checked safely in Omega. Target/reference
policy and opcode retirement remain pending; aggregate counts are unchanged.

[Expanded canonical byte Index validation](../aml/byte-index.PORT.md) checks the existing byte_storage constructor without adding a second implementation.
All 117 complete-store checked pairs, three bounded constant pairs and 44 actual
public Index observations pass, covering two-slot allocation, failure preservation,
transparent references and fresh field/reference identities. Source evaluation,
target Store and opcode retirement remain pending; source counts are unchanged.
