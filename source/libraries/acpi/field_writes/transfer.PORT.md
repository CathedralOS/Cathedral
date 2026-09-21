# One-payload normal Field write sequencing

Status: **tested**. All 102 behavior/control pairs and eight selected unchanged
chunk/bulk regression pairs pass with exact source and binary verification.
This adds a detached continuation
for one already-converted payload. ACPI-005 and whole upstream anchors stay open.

`transfer.omg` and `transfer_model.omg` adapt the ordering in rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:2612`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2612),
`do_field_write`, copyright 2018 Isaac Woods, MIT OR Apache-2.0. The existing
[inventory](inventory.json), [notices](../../../../THIRD_PARTY_NOTICES.md),
[licenses](../../../../licenses/rust-osdev/acpi/) and
[library charter](../../CHARTER.md) apply. The original assembly functions and
their arithmetic bodies are unchanged.

[ACPI 6.6 §19.6.48](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#field-declare-field-objects)
defines native access, surrounding-bit updates and locks. The continuation
retains the pinned read/merge/write order while using the corrected canonical
geometry and payload algorithms. Existing source-conversion interpretations
belong to [field_sources](../field_sources/PORT.md); this API receives one
field-sized byte vector and performs no AML conversion or repeated source Store.

## State and ordering

`transfer::begin` admits complete normal-Field geometry, exact logical payload
length, then the lock requirement, before returning any request. An unmet Global
Lock returns failure. Bank/Index, BufferAcc, unsupported metadata and invalid
footprints retain canonical admission errors. The plan, field metadata and all
256 initialized payload bytes are copied into a `WriteTransfer`.

`progress` returns Failure, Pending or Finished. Each Request has an explicit
Read/Write kind, caller correlation, monotonically increasing serial, chunk
index, region-relative byte offset, native width and write value. A read request's
value is zero. Reading progress does not mutate the continuation.

Each partial Preserve chunk issues one Read. Its matching Word completion feeds
the existing package-private `write::chunk_value` kernel and issues that chunk's
Write immediately. Only a matching Written acknowledgement advances to the next
chunk. Complete chunks and WriteAsOnes/WriteAsZeros omit the read. Geometry is
retained from admission, avoiding a full field re-plan for each acknowledgement.
The same merge body serves bulk, selected-chunk and continuation APIs.

Matching includes every request coordinate, kind and value. Wrong coordinates,
wrong response variants, duplicates and responses to a closed continuation are
rejected without changing any state. A matching provider failure is accepted
and terminal. Finished is reached only after all native writes are acknowledged.
The counters report successful read completions and acknowledged writes; they
never imply that a failed provider call had no effects. Prior acknowledgements
are retained without retry or rollback. Default initialization is a closed
InvalidState failure.

The inherited bounds are 1–2048 field bits, 256 payload bytes and at most 257
chunks. The conservative request bound is 514, permitting at most one read and
one write per chunk. All indexing and counters remain bounded. The public data
is an ordinary copyable model, with `begin` followed by `progress`/`complete` as
its supported construction and mutation path; it is not an unforgeable session.

## Integration boundary

This package invokes no provider and represents no address-space, region,
namespace or live device identity. Relative requests and caller correlation are
ordinary numeric data. A supplied word is not a fresh physical observation, and
an acknowledgement is not a hardware receipt. Locks, suitable register semantics,
physical placement, provider grants, synchronization and partial-effect recovery
remain the [adapter's](../ADAPTER.md) obligations. Copying a continuation grants
no access and cannot recreate a live device/session occurrence.

The API has no ObjectStore, runtime, target or expression result. Later AML
integration must retain the pending retirement operation, both Divide targets
where applicable, and source-sequence progress. For ordinary Field access types,
Store contributes the converted data written under
[§19.6.132](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#store-store-an-object),
without an extra field read. BufferAcc's handler-modified result lies outside
this profile. Existing earlier AML and acknowledged device effects cannot be
rolled back by a later failure.

## Evidence

[Owned tooling](../../../../tools/ports/acpi/field-write-transfer/README.md)
generates independent full request traces for native widths, update rules, both
Integer sizes, aligned/partial shapes, 2048-bit capacity, admission and all
failure positions. Rejection cases compare the complete continuation, including
Field metadata, all 257 geometry slots and the 256-byte payload. Controls alter
the expected payload tail and must fail the complete-state comparison.

The [focused receipt](../../../../tools/ports/acpi/field-write-transfer/focused-verification.json)
passed five pairs in 119.344 seconds with 69 exact source/tool hashes and maximum
fuel 920,301. It includes the 2048-bit capacity, partial Preserve, wrong response
variant, late provider failure and default initialization, with complete state
comparison. This selected evidence is separate from the full corpus run.

No new public Rust, constant-evaluator, native Omega or hardware observation is
claimed. Earlier arithmetic and public Rust receipts retain their exact
historical inputs.

The full run checked 11 authored packages sequentially in 693.833 seconds
(693.792 seconds summed package time), with 69 bound source/tool inputs and
maximum evaluator fuel 920,301. The eight selected unchanged regression pairs
passed in 145.500 seconds across two packages, maximum fuel 788,840. These
regressions cover default failures, complete writes without old words, maximum
field capacity and maximum unsigned payload-length rejection in both public
assembly forms. Every retained positive returned zero and each control returned
one. The source-bound receipt verifiers also check generated text, entry names,
input stability and the exact runner binary.
