# Resumable Field-target retirement

Status: **transcribed; verification running**. This branch adds executor
continuations for Store and scalar result targets. The 34 authored whole-state
behavior/control pairs have not yet passed. Provider-facing pipeline composition
and bytecode evidence remain pending. ACPI-005 stays open.

## Source and contract

This composition extends the existing adaptation of rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5),
`src/aml/mod.rs` (`do_store`, scalar retirement and `OpInFlight`), copyright
2018 Isaac Woods, MIT OR Apache-2.0. The existing
[generic inventory](generic-inventory.json),
[notices](../../../../../THIRD_PARTY_NOTICES.md),
[licenses](../../../../../licenses/rust-osdev/acpi/) and
[library charter](../../../CHARTER.md) apply. No complete upstream anchor closes.

The implementation was preceded by review of the
[adapter contract](../../ADAPTER.md), [named Store](named-store.PORT.md),
[single-payload write continuation](../../field_writes/transfer.PORT.md),
[source sequence](../../field_sources/PORT.md), pinned Rust bodies and ACPI 6.6
[Divide](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#divide-integer-divide),
[Store](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#store-store-an-object)
and [conversion rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules).
Divide writes its remainder before its quotient and retains the quotient as its
expression result. The completion owner selects the admitted Store result; an extra hardware read
is not part of retirement. The isolated [write pipeline](../../pipeline/field-writes.PORT.md)
documents its admitted-source compatibility policy for complete multi-payload
writes, including an empty String. This helper does not select a last payload.

## Representation and execution

`Frame.field_writes` defaults to false. Existing synchronous entry points keep
their prior unresolved-Field behavior. The new `start_write_runtime` enables
the mode and nested frames inherit it. `advance_write_runtime` charges one AML
turn and pauses while a `DeferredWrite` is present. Polls and completion calls
charge no additional turn. The cumulative 1024-turn limit remains intact.

`DeferredWrite::Field` retains the selected object, source operand, expression
result, Store-result policy, optional second target and its already-computed
Integer. The original operation can therefore be popped once without losing
the quotient write or recomputing either operand. Before suspending, a private
Frame copy verifies that the eventual parent contribution has room.

`write_retirement` identifies direct named Fields and explicit-reference Arg
targets without changing their bindings. Other destinations use the existing
generic or Integer target bridges. Local replacement, named conversion,
CopyObject and explicit ToInteger replacement retain their prior policies.
This work adds no general reference opcode or implicit physical access.

`complete_field_write` and its Runtime wrapper accept the completion owner's admitted
Store-result operand, validate it and resume the remaining target or final contribution.
Scalar results retain their computed value independently of the completed Field
payload. A second target failure retains prior local/namespace effects. The
future provider owner must also retain earlier external acknowledgements.
Completing an absent continuation fails without publication. This is an ordinary
internal data API: request identity and provider acknowledgement matching belong
to the provider-facing owner, not this helper.

The Runtime wrapper latches a late error after the pending write has been
consumed, so subsequent advances retain that failure and cannot execute later
AML. An invalid Store completion operand leaves the pending write intact and
can be corrected without consuming fuel. Ordinary non-Field Store targets retain
the legacy source operand identity for their expression contribution, including
an object-backed Integer stored to a Local.

## Remaining integration

There is no provider Session in this slice. Field metadata, source conversion,
region span, physical footprint, access profile, locks and full request identity
must be admitted before any effect by the pending pipeline work. Result object
allocation and graph quotas remain its obligations. Increment/Decrement of a
Field needs a separate target-read continuation; general value reads and Mid
result targets must reconcile with their independently tested branches.

The new runtime hooks currently describe a write-only execution mode. Combined
read/write scheduling must pause on either continuation, preserve both members
in staged Frame/Runtime copies, inherit both modes on calls, and derive a request
bound that includes two Divide targets and repeated source payloads. It must not
reuse a read-only session's request limit without that derivation.

## Verification

The [owned fixture tool](../../../../../tools/ports/acpi/interpreter/field-write-execution/README.md)
compares every ObjectStore and Frame member after each retirement/completion.
The new continuation comparator includes both target/result alternatives and
all retained scalar coordinates. Controls change an inactive lookup-cache slot.
The 34 cases span both Integer sizes, converted Store results, invalid and
duplicate completion, disabled mode, full parent, Arg references, scalar result
identity, both Divide targets, local/Field combinations and late target failure.
No public Rust, constant-evaluation, native execution or hardware result is
claimed by unexecuted expectations.

Review of the first transcribed checkpoint `2d61769` found the missing Runtime
error latch and changed ordinary Store contribution described above. Its small
two-pair check was deliberately stopped before claiming any result. The updated
34-pair corpus includes eight whole-Runtime cases: late failure followed by
advance, invalid completion followed by a valid retry, pending polls and a
second suspended Divide target. The full run binds the corrected sources.
