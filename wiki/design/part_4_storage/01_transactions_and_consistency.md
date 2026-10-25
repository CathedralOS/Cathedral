# Chapter 01: Transactions & Consistency

> Callers list expected revisions and desired changes. The storage service commits
> the admitted batch together or not at all.

## Accepted scope

The [atomic batch contract](../../spec/storage/atomic_batches.md) owns the accepted
transaction surface: N explicit revision prerequisites and M mutations within one
storage service's transaction domain. It is ordinary service machinery, not a new
Omega transaction construct or a kernel-wide coordinator. The
[storage foundation proposal](../../proposals/0002_dbfs_storage_foundation.md)
retains candidate engine choices and experiments; existing bounded transactions
do not establish the complete public batch API.

The motivating case is a multi-file edit. An editor reads a declaration and its
callers, prepares replacement bytes, and asks to replace both files only if both
still have their observed revisions. A second case checks several input files
but publishes one derived output. The service does not need to understand the
source language or application invariant.

## Explicit prerequisites, not inferred constraints

The caller chooses which observations justify its writes. The service validates
those exact revisions, including prerequisite objects that are not written, and
coordinates validation with publication so no conflicting mutation slips between
them. Ordinary storage validity and authority checks still apply.

This is stronger than separate N=1 replacements: the first replacement cannot
succeed while a later one conflicts and leaves half an edit installed. It is not
automatic tracking of all reads or a promise that arbitrary queries remain true.
For example, checking each file returned by a query does not detect a newly
created matching file. General predicate/query-aware serializability is deferred
until a concrete customer requires it and its conflict contract is selected.

Copy-on-write, immutable backing and snapshot reads do not themselves provide
that broader isolation. Nor does a local borrow establish exclusive access
against independently authorized service clients.

## Preparation and bounded commit

Callers prepare optimistically without holding transaction-wide locks. The
service validates a stable request, stages payloads and reserves resources before
publication. Mutable shared buffers must not let the caller replace the validated
request underneath the service.

Caller funding does not isolate shared commit work. Admission must account for
validation, commit resources, retained versions and interference with other users.
The service can refuse an oversized or underfunded request; it cannot quietly
split an atomic batch. Slow preparation often causes a revision conflict, but
unchanged inputs can leave a very large batch valid, so conflict is not a resource
bound.

One transaction domain may contain multiple authorized realms. This does not
grant cross-realm authority or require two-phase commit. A RAM mutation, function
return or language transition is not durable commit. Backing must be recoverably
durable before committed references point to it, and reclamation must preserve
all retained and actively read state. Exact engine, locking and recovery details
remain implementation choices under these obligations.

## Failure and recovery

Confirmed conflict or pre-commit refusal changes none of the requested objects.
The caller chooses whether to reread and retry. A lost reply or uncertain commit
error is different: it may follow a successful commit. Request identity and
authorized outcome reconciliation must distinguish that uncertainty from refusal,
with explicit funded retention and recovery. Do not blindly resubmit mutations
or silently rebase them onto new input revisions.

The [change-watch contract](../../spec/storage/change_watches.md) requires durable
capture to commit required event information with mutations. Reserve capture
capacity before commit and deliver records asynchronously afterward. Recovery
retains committed changes even when delivery never occurred; aborted transactions
are not completed-change events. Live-only watches instead use caller-funded
buffers with explicit overflow. Neither mechanism implies retaining old file bytes.

## Boundaries and deferred work

- General query-aware serializability and implicit read-set capture are not
  prerequisites for the explicit batch. Raise a new owner question only when an
  actual customer exposes a missing semantic choice.
- Independently committing services, external effects and kernel authority changes
  are outside this storage transaction. No universal commit/abort mechanism or
  distributed transaction manager is selected.
- [Installation and activation](../../spec/execution/updates.md) remain separate:
  atomically publishing installed storage does not atomically migrate live programs.
- Replication merge rules and CRDTs require meaningful structure and their own
  contract. They are not automatic conflict resolution for arbitrary file bytes.
- Transactional outboxes or compensation may be useful for future cross-service
  workflows, but they do not turn an irreversible external effect into a local
  rollback or promise exactly-once delivery.

## Implementation path

The [batch outcome](../../../TASKS_RUST.md#explicit-atomic-storage-batches) carries
the service/client API, invalid controls, admission and recovery to usable
completion. Reuse the existing object engine and protocol boundary rather than
creating an OS-wide transaction framework. This document ratifies no engine choice,
numeric request limit or new source-language syntax.
