# Concatenate execution

Status: **transcribed**. Opcode `0x73` now has two value operands, one target,
and an isolated retirement implementation. The authored 110 behavior/control
pairs comprise 86 complete-state retirement cases and 24 actual AML cases.
An eleven-pair focused checked validation is running; it has not yet completed. ACPI-005 and complete upstream anchors
remain pending; this is not a passing integration claim.

## Basis and scope

`concat_execution.omg` and `aml/inline_concat.omg` adapt the composition in
rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:2171`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2171),
`do_concat`, copyright 2018 Isaac Woods, MIT OR Apache-2.0. The existing
[generic inventory](generic-inventory.json),
[notices](../../../../../THIRD_PARTY_NOTICES.md),
[licenses](../../../../../licenses/rust-osdev/acpi/) and
[library charter](../../../CHARTER.md) apply. The [generic execution contract](generic.PORT.md),
[adapter contract](../../ADAPTER.md), [basic Concatenate](../../aml/object-concat.PORT.md),
[description composition](../../aml/object-concat-described.PORT.md), pinned
source and ACPI 6.6 were reviewed before implementation.

[ACPI 6.6 §19.6.12 and Tables 19.30–19.31](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#concatenate-concatenate-data)
make the left operand select the right conversion and result type. The existing
canonical helpers retain primary hexadecimal String conversion, Buffer text
formatting and Integer width, including documented differences from the pin.
This extension does not silently replace those rules with the pin's decimal/raw
UTF-8 behavior.

The synchronous decoder admits preloaded basic values and Packages; it does not
make all direct description-helper values reachable through ordinary AML.
Methods are invoked, and Device/Region/Uninitialized and unsupported Field operands
retain existing admission behavior. Direct retirement can use the descriptions
accepted by the canonical composition. Integer × description remains that
helper's documented local exclusion, not a claimed normative ACPI rejection.
General reference bytecodes, dynamic byte literals and combined Field reads/writes
remain separate work. These limits prevent any whole-Concatenate completion claim.

## Result allocation and targets

Retirement resolves transparent carriers left then right under the existing
64-inspection bound and normalizes Integers to the active width. Explicit RefOf
and Index values retain the underlying conversion helper's unsupported policy.
Canonical byte admission follows resolution, then target preflight, result
allocation and target application. Earlier operand evaluation effects remain.

Stored-object pairs delegate to `object_concat_described`. `inline_concat`
supplies two-Integers, Integer-left and Integer-right entry points without
allocating temporary caller object slots. Its conversion and append arithmetic
reuse the existing pure helpers. Complete byte inputs are admitted before
combined capacity. String results exclude the terminator; Buffer conversion of
a nonempty String includes it. Every result contains 256 initialized bytes with
zero unused tails.

The expression result is a fresh owned String or Buffer. Target selection checks
the original arena before allocation so a stale ID equal to the old object count
cannot become valid. Ordinary target conversion, Local/Arg binding, allocation
and parent contribution occur in one private ObjectStore/Frame stage; failure
publishes none of those changes. The result retains its own type and identity
when the optional named target uses a different conversion. Null still creates
and contributes the result. A new Local/Arg cell may need a second slot.

The additive `write_retirement::retire_result` keeps this non-Store expression
result through an optional Field write. Named Field preflight admits an existing
canonical Field only when `field_writes` is enabled; the continuation then checks
parent room and retains both source and result. Arg reference Field targets use
the same continuation. Completion contributes the original result, independent
of the supplied Store-completion operand. Disabled mode retains UnresolvedRegion;
Debug retains UnresolvedService. Publication of a pending result does not imply
a device write or validate Field metadata: the provider Session owns those checks
and later effects. This branch has no live provider integration.

The inherited limits remain 64 objects, 32 namespace entries, 256 result bytes,
1024 source bytes and the bounded executor. Result-graph quotas apply at the
Program boundary. A later external or quota failure cannot roll back earlier
method or provider effects.

## Evidence

[Owned fixtures](../../../../../tools/ports/acpi/interpreter/concat-execution/README.md)
use independent Python byte/integer/hexadecimal operations for expectations.
Complete-state cases compare every ObjectStore/Frame member, including the Field
continuation and inactive backing/cache slots, before and after completion.
Cases cover all basic pairings at both widths, stored and inline Integers,
descriptions, full/empty/overflow results, late malformed bytes, self-targets,
ordinary and reference bindings, stale identities, exhausted arenas, full parents,
and Field result identity. A review found and corrected named Field rejection
before continuation; enabled and disabled named cases are retained.

The 24 AML bodies exercise opcode shape and dispatch through the real loader and
Program executor, including all nine basic pairings, Local targets, String
self-targets and nested results. Controls alter inactive expected state and must
fail the assertion body. Their presence is not evidence of successful execution.
The immutable Omega runner is used without compiler changes. Public Rust
Concatenate observations in prior components remain historical; no new public,
constant-evaluation, native or hardware result is claimed here.

The initial nine-pair run at `5b32747` was stopped before any result because
review found a record-type mismatch: the new resolver needs `OperandResult`,
not the existing Value metadata carrier. Review also corrected the malformed
String expectation to Encoding and the transparent-cycle expectation to the
existing adapter's InvalidState mapping. The production error mapping was not
changed to satisfy tests. The updated focused selection includes both errors.
