# Explicit atomic storage batches

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | None |
| Last reviewed | 2026-10-25 |

## Scope and ownership

A caller may submit N explicit revision prerequisites and M storage mutations as
one transaction. The storage service checks those prerequisites and publishes all
mutations together, or none. N and M need not match: prerequisites may name input
files that are not being changed. The storage service does not infer application
invariants from file contents or merge arbitrary bytes to resolve conflicts.

This contract covers one storage service's transaction domain. Authorized objects
may span realms within that domain; realm boundaries still enforce authority.
It does not coordinate independently committing services, kernel capability tables
or external effects. No new Omega syntax or kernel transaction manager is required.

## Caller-supplied request

The request contains stable object references with expected revisions, the desired
mutations and their payloads, and an identity for outcome reconciliation. For example
(protocol illustration, not source syntax):

```text
preconditions:
    api.omg  at revision 12
    main.omg at revision 8
changes:
    replace api.omg with prepared declaration changes
    replace main.omg with prepared caller changes
```

Another request may check several source revisions and publish just one output.
Only the explicitly listed revisions are prerequisites. Ordinary mutation validity
and authority checks still apply; omitting a prerequisite grants neither authority
nor permission to violate the storage service's structural rules.

Revision comparison identifies the exact object incarnation and observed state
covered by that revision. Reused object slots or change-then-restore must not silently
satisfy an old expected revision. A byte-content hash alone does not establish
unchanged hierarchy, metadata or authority. Concrete token encoding is engineering
work; the API must state what each revision covers.

Callers need authorized observation for prerequisites and the corresponding
mutation rights for changes. Observation does not confer write authority, and a
revision token is not a capability. Checking a hidden object's revision must not
become an existence or content oracle. Authority validation is coordinated with
publication under the service's grant/revocation rules; staging is not permanent
authorization to commit later.

Lists may originate in stack, heap or other caller-owned storage. The service must
consume a stable validated request and stable payloads: a hostile shared-memory
writer cannot change prerequisites or mutations after their validation. Copying,
sealing or an enforced ownership transfer may establish that stability; borrowing
an untrusted mutable buffer without enforcement does not.

## Preparation, admission and commit

Callers compute optimistically without transaction-wide locks held on their behalf.
The service can validate, stage bytes and reserve resources incrementally before
publication. Slow preparation can lead to stale prerequisites; it is not permission
to block other writers indefinitely.

Requests are caller-funded and service-admitted, not unlimited. Admission bounds
request size, validation/commit work, retained storage and interference with other
clients. Scheduling another task does not release a lock, storage bandwidth or a
version pin. The service may reject insufficient funding, capacity or an oversized
batch before mutation. It must not silently split one batch into partial commits.
No universal numeric limit is selected here; admitted bounds must be explicit.

Final prerequisite validation and publication are coordinated against competing
mutations. No write may slip between a successful check and the publication it
justifies. A mismatch refuses the whole batch. On success all requested changes
publish as one commit, with durable backing established before references to it
become committed. Recovery must not expose a partially applied batch. Staging may
leave private reclaimable material, but not partially published requested changes.

Readers using separate calls can straddle a commit; atomic publication does not
promise a snapshot spanning arbitrary independent reads. The guarantee here is an
indivisible mutation, not automatic tracking of an application's reading history.

Required [durable change events](change_watches.md) belong to that same storage
commit. Capture reservations and policy admission are part of its resource checks.
Neither a language return nor a RAM write substitutes for durable publication.
Exact locking, validation indexes, recovery format and batching remain engineering.

## Outcomes and retry

A confirmed conflict or pre-commit admission refusal means none of the requested
mutations committed. The caller decides whether to reread, recompute and retry;
the service must not silently rebase the request onto different input versions.

A lost reply or uncertain commit error is not a confirmed refusal. A committed
batch might already exist. The service provides request identity and an authorized
outcome-reconciliation mechanism tied to that exact request, preserving the commit
decision through recovery within the admitted reconciliation lifetime. Reusing an
identity for different contents must not select an old success for new mutations.
An unknown outcome is never reported as definitely uncommitted; blind retry is
not the recovery protocol. Outcome records and their retention must be funded and
their lifetime stated, rather than promising infinite deduplication history.

## Deliberately outside this decision

There is no implicit read-set capture, query/predicate conflict tracking or general
serializability promise for arbitrary application logic. Checking revisions of all
returned files does not establish that a query would return no additional files.
Applications need an explicitly supported prerequisite covering that dependency,
or must not claim it is protected by this batch. General query-aware transactions
need a concrete customer and a separate contract, not a silent strengthening of
this interface. Distributed commit and automatic CRDT conflict resolution are
also excluded.

## Implementation boundary

The [object library](../../../source-rs/platform/libraries/database-objects/README.md)
has revision checks and bounded engine transactions; the
[storage service](../../../source-rs/platform/services/storage/README.md) exposes
individual operations. Those building blocks do not establish the generic public
N-prerequisite/M-mutation contract or durable request reconciliation.
The [atomic batch outcome](../../../TASKS_RUST.md#explicit-atomic-storage-batches)
tracks remaining end-to-end acceptance. This ratification changes no runtime code.
