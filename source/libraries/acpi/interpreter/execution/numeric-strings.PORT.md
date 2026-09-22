# Explicit numeric String opcode execution

Status: **transcribed; checked Omega execution pending**. Opcodes `0x97` and
`0x98` now dispatch ToDecimalString and ToHexString through
`numeric_string_execution.omg`. The existing formatting helpers are tested at
their detached boundary; that evidence does not execute this adapter. Authored
160 bytecode and 171 complete Store/Frame retirement scenarios, plus 315 retained
executor regressions, require their own exact-source checked results. ACPI-005
and aggregate counts remain pending.

The original focused AML batch failed to parse a test-helper local named
`block`. An isolated original/renamed probe confirms the identifier issue, and
the exact failing package is retained under the tooling's
[diagnostic archive](../../../../../tools/ports/acpi/interpreter/numeric-string-execution/diagnostics/block-local/).
The candidate renames that local to `snapshot`, preserving all production
inputs, fixture rows and expectations. This does not establish an AML execution
pass; the original retirement batch and corrected checked runs remain pending.

This modified composition maps rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:2096`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2096),
`do_to_dec_hex_string`, copyright 2018 Isaac Woods, MIT OR Apache-2.0. The
[source inventory](generic-inventory.json),
[translation notices](../../../../../THIRD_PARTY_NOTICES.md) and
[licenses](../../../../../licenses/rust-osdev/acpi/) retain provenance. Opcode
shapes are in `operator_specs.omg`; local wrapper states in `retire.omg` call
the adapter. The generic decoder, target bridge and canonical formatters are
unchanged. This remains outside Cathedral production build roots.

## Source conversion

[ACPI 6.6 §§19.6.139–140](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#todecimalstring-convert-data-to-decimal-string)
defines the two explicit String conversions. Their
[AML encoding](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html#expression-opcodes-encoding)
supplies one evaluated source and one target. The adapter composes the existing
[canonical numeric String policy](../../aml/object-numeric-strings.PORT.md)
after transparent Named/Local/Arg reference resolution.

Inline Integers are formatted directly without allocating an operand object.
Canonical Integers follow the same selected 32/64-bit normalization. Strings
are copied after complete ASCII/NUL-free backing validation; numeric syntax
is irrelevant for String identity. Buffers format every logical byte, including
zero, into comma-separated values. Empty Buffers produce empty Strings.
Decimal uses minimal digits. Hex Integers use `0x` and minimal uppercase digits;
hex Buffer bytes use `0x` and two uppercase digits. These existing prefix and
padding choices follow the pin beyond the primary clauses' presentation detail.
Implicit fixed-width/space-separated String conversions are not used here.

The result is a detached byte snapshot with a logical length at most 256 and
fully initialized zero padding. Complete source validation precedes formatting
capacity admission. Hex Buffer length 51 fits and 52 overflows; decimal length
depends on the byte values. Exact 256-character output is admitted. Owned
nonlogical source tails are ignored and cannot leak into result padding.

Uninitialized operands retain the execution Uninitialized outcome. Reference,
storage and formatting errors use the existing conversion mappings. Source
RefOf/Index carriers presented directly are unsupported; ordinary argument
decoding retains its own reference policy before retirement. BufferField,
Field Unit and service-bearing source conversion remain outside this adapter.
No provider, synchronization or hardware callback is introduced.

## Publication and rollback

[Implicit result conversion §19.3.5.5](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-result-object-conversion)
excludes explicit conversions from ordinary target coercion. The adapter uses
the canonical copy path: named targets retain their object ID while becoming
String, and Local/Arg targets retain the existing binding and explicit-reference
redirection rules. This differs from the pin's `do_store` target behavior.
Null discards only the target write. Debug returns UnresolvedService; protected
targets keep their existing admission failures.

Every successful operation allocates an independent owned String result, even
for String input or a Null target. The expression contributes that fresh result
ID, independently of any target cell. Target IDs and retained cell bindings are
admitted against the original store before allocation, so a dangling target
cannot become valid by naming the new result. Source conversion/formatting is
completed before that target preflight.

Allocation, target publication and parent contribution run on a private
ObjectStore/Frame copy. Only complete success publishes both. A late parent
failure, second required cell allocation, invalid target or capacity error
therefore preserves the complete original store and frame. Effects of earlier
AML statements and source-expression evaluation precede this retirement boundary
and remain visible on later failure. The profile remains 64 canonical objects,
32 namespace entries, 256 byte backing and a 1024-byte source snapshot.

## Evidence

[Owned tooling](../../../../../tools/ports/acpi/interpreter/numeric-string-execution/README.md)
distinguishes actual loaded AML, complete Store/Frame retirement comparisons,
the 315 retained Mid/executor regression pairs and public pinned Rust observations.
Changed-expectation controls must execute along with every positive case.
Checked receipts bind the exact source inputs, generated modules, selected
entries, driver/build text and runner binary. Source/host-generation audits
alone do not establish compilation or execution. Public Rust observations retain
raw target/width/error differences and do not execute this Omega adapter.
No constant-evaluation, native Omega or hardware result is claimed here.

The current [public receipt](../../../../../tools/ports/acpi/interpreter/numeric-string-execution/public/verification.json)
contains 124 actual observations, independently verified against all exact
fixture/tool inputs and the rebuilt binary. It retains 82 result/type/named-state
agreements and 42 nonagreements: 14 primary-error cases without claimed error
equivalence, 20 value/state differences, four evaluation errors and four load
panics from initializer-dominates Buffer construction. Every observation records
zero forbidden callbacks and one inert mutex. The 36 explicit omissions are
34 canonical store edits and two Debug cases. This probe cannot compare
Cathedral object IDs or nonlogical byte padding.
