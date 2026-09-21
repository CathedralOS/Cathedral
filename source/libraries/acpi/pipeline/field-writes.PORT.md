# Retained normal Field write pipeline

Status: **transcribed; checked validation running**. `field_writes.omg` composes
AML target retirement, canonical source sequencing and native write requests.
The 68 authored behavior/control pairs are expectations, with an eight-pair
selection running. No checked, public Rust, constant, native or hardware result
is yet claimed for this composition. ACPI-005 remains open.

## Basis and result policy

This is original Cathedral composition over the existing MIT OR Apache-2.0
adaptation of rust-osdev/acpi at
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5),
copyright 2018 Isaac Woods. Existing
[notices](../../../../THIRD_PARTY_NOTICES.md),
[licenses](../../../../licenses/rust-osdev/acpi/),
[library charter](../../CHARTER.md) and [adapter contract](../ADAPTER.md) apply.
No whole upstream anchor is classified as translated by this adapter.

Before implementation, the source review covered the charter, adapter, pinned
`do_store`/`do_field_write`, [named Store](../interpreter/execution/named-store.PORT.md),
[write retirement](../interpreter/execution/write-retirement.PORT.md),
[source sequence](../field_sources/PORT.md),
[single-payload continuation](../field_writes/transfer.PORT.md), and ACPI 6.6
[Store](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#store-store-an-object),
[conversion rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
and [Divide](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#divide-integer-divide).

For this ordinary-Field profile, Store contributes its admitted source after the
entire source sequence. Integers are normalized to the executing frame's width;
Buffer/String results retain the admitted object identity. Empty String completes
without a provider request and still contributes that source. This is an explicit
compatibility policy: the specification's data-written wording does not explicitly
select the last payload or define a multi-piece result. Inspection of ACPICA's
[AML Store result default](https://github.com/acpica/acpica/blob/master/source/components/executer/exoparg1.c)
and [ordinary Field writer](https://github.com/acpica/acpica/blob/master/source/components/executer/exfield.c)
supports retaining the source when the writer supplies no special result.
No ACPICA implementation was copied or executed. The pinned Rust implementation's
single-pass Buffer behavior and rejected String conversion do not prove this
repeated-source profile. No extra Field read or result object allocation is used.
BufferAcc handler-modified results remain outside this scope.

Scalar operations retain their computed expression value. Divide acknowledges
the remainder target before admitting the quotient target, then contributes the
quotient once. A later target failure preserves all earlier acknowledged effects.
Ordinary non-Field Store semantics stay in the existing target bridges.

## Session and requests

`begin_limited` owns a prepared Program and admits result-quota and AML-fuel bounds
before arguments or execution. Start admission failure is closed; Capacity is
also represented as `Fault::Execution(Capacity)`. An unprepared/default session
has InvalidState behavior. Program source and metadata are ordinary public data;
clients must preserve the snapshot while using `advance` and `complete`.

`advance` runs bounded AML scheduling turns until a write suspends, execution
finishes, or the requested quantum is consumed. Retained writes are prepared
before checking the remaining AML fuel. Provider completions and polling spend
no AML turn, including a second Divide write after the charged retirement.

Before the first request, preparation verifies canonical normal Field binding,
declaration kind and retained source spans, OperationRegion identity, SystemMemory
space, geometry, absent Global Lock requirement, native width, physical base
alignment and nonoverflowing exclusive footprint. Bank/Index and unsupported
profiles retain explicit errors. Source admission uses the canonical sequence
kernel, validates the complete backing, and retains stable source identity.
Buffer payloads repeat by field width, String payloads by character, and empty
Buffer supplies one zero payload. Empty String supplies none.

Each payload owns one `WriteTransfer`: partial Preserve reads immediately precede
the corresponding native writes; full chunks and other update rules omit reads.
Each request includes session correlation, a globally increasing serial, retained
unit, Field and region IDs, space, base and payload ordinal, plus all seven inner
request coordinates. Matching checks every coordinate before forwarding to the
single-payload continuation. Inner serials may restart with each payload; the
outer serial distinguishes occurrences. Wrong identities, response variants,
stale failures and duplicate acknowledgements must leave the entire session
unchanged.

Successful Word and Written responses increment separate read and acknowledged
write counters. Matching provider failure is accepted and terminal, retaining
its cause and all prior counters. A failed provider call can have effects that
these counters cannot describe. There is no automatic retry or rollback.
A final result-quota failure removes the returned value while retaining earlier
execution and acknowledged writes.

The supported bounds remain 1024 AML turns, two targets per retirement, at most
2048 payloads per target and 514 native requests per payload. Conservative session
limits are 4,194,304 completed payloads and 2,155,872,256 native requests. Each
`complete` handles one response; no request array of that size is allocated.

## Evidence and remaining work

The [owned fixtures](../../../../tools/ports/acpi/pipeline/field-writes/README.md)
author actual loaded AML Store/Divide methods. Independent Python bit arithmetic
predicts complete memory and request traces for both Integer widths, all native
widths/update rules, repeated Buffer/String writes, empty sources, wide Fields,
provider failures and late target/result-quota errors. All 512 memory bytes and
512 initialized trace slots participate in comparisons. Twenty additional cases
compare complete sessions on rejected responses: Program source/store/definitions,
all Runtime frames, result, every transfer/geometry/payload slot and all counters.
Controls alter an inactive trace slot or retained source byte.

The immutable pinned Omega runner checks authored bodies and interprets them;
its compiler is not modified. The checker binds source/tool hashes, generated
text, exact entries, build text, runner binary and before/after stability. Until
a passing receipt is verified, these remain unexecuted expectations.

Live provider invocation, grants, locks, mappings, volatile-register semantics and
hardware receipts remain adapter work. Combined Field reads/writes, read-before-write
Increment/Decrement, independently developed Mid targets, wider regions and
BufferAcc are still pending. This write-only Session does not reconcile the
separate read-session continuation or claim either branch's evidence.
