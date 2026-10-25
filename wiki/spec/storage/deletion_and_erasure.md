# Reference deletion, retention and scoped erasure

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | None |
| Last reviewed | 2026-10-25 |

## Service boundary

Deletion is a storage-service operation with an explicit retention policy, not a
kernel syscall or a new Omega construct. The conceptual shape is
`delete(reference, retention_policy)`; this does not freeze a wire encoding or
source spelling. The caller must have authority over the reference and over any
requested removal of its retained recovery paths. An address, object number or
content hash is not that authority.

The operation acts on the specified authorized reference and declared retention
scope. It does not search for and destroy all occurrences of the same bytes.
Independent copies remain independent under the
[copy-on-write contract](realms_and_objects.md).

## Hidden backing and independent ownership

Physical backing is an implementation detail. Two independently owned files may
share immutable bytes without either owner managing or learning that fact:

```text
A deletes its reference -> deletion can succeed
B retains its independent reference -> B remains usable
last retained consumer releases backing -> backing becomes reclaimable
```

A write through A must not change B. Sharing backing grants no additional access,
and deleting A cannot revoke B. Identical content is not evidence of shared mutable
identity or permission to operate on another realm. Each reference may carry its
own access and retention obligations even when physical storage is shared.

Deduplication need not introduce a user-visible sharing-domain object. Internally
it must still preserve encryption, authority, retention and funding obligations;
this decision does not choose a cross-realm key scheme or promise universal
deduplication. Hash lookup and placement must not become ambient access APIs.
No new timing-side-channel guarantee is established here.

## Retention policies and success

The operation distinguishes these promises:

- **Ordinary deletion:** remove the reference, honoring the applicable realm's
  history policy. Retained history may deliberately permit recovery. This is not
  a default promise of cryptographic destruction.
- **Deletion without history:** additionally remove the managed recovery paths
  covered by the request. Independently retained copies remain outside its scope.
  Logical removal does not itself promise immediate physical overwrite or
  cryptographic destruction of residual storage.
- **Cryptographic erasure:** additionally make the specified reference's covered
  cryptographic access and recovery paths unusable. Success requires actual key
  machinery that enforces this, not merely deleting a directory entry or checking
  that no visible file points to the bytes.

These are policy choices at one service boundary, not three unrelated storage
subsystems. Default history duration and thinning schedules remain separate policy;
no fixed schedule is ratified here. The service must identify the scope of an
erasure promise rather than presenting a partial operation as global destruction.

An independently owned surviving file is not failure of scoped erasure. Conversely,
an old managed copy of the erased reference or its key material that still restores
the covered access path prevents a cryptographic success claim. Managed snapshots,
recovery records, key backups and cached material must be accounted for within the
advertised scope. Plaintext or independent copies already obtained outside that
scope cannot be recalled by deleting this reference.

If the requested guarantee cannot be provided, the service refuses it explicitly;
it must not silently downgrade cryptographic erasure to ordinary deletion. It
must validate support and authority before destructive work, and report any
uncertain or incomplete completion honestly rather than claiming success.
Request identity and outcome reconciliation follow the
[storage commit boundary](atomic_batches.md#outcomes-and-retry).

## Realm encryption is not per-reference erasure

Realm locking/unlocking controls access to encrypted storage. The existing
[realm-sealing design](../../design/part_1_authority/04_secrets_and_keys.md#realm-sealing-envelope-encryption-with-n-wrapped-keys)
describes a realm data key with multiple wrapped unlock paths. That alone does not
give every file reference a separately destroyable cryptographic path. Deleting
one entry while retaining a key able to recover it establishes logical deletion,
not the stronger promise.

This contract requires the guarantee, not a particular per-file or per-reference
key hierarchy. A provider without suitable cryptographic machinery rejects that
policy. Independent owners may retain their own authorized paths to shared
backing, provided the erased path cannot itself be restored within its scope.
Erasing an entire realm and erasing one reference are different scopes; neither
implies authority to destroy another owner's copy.

## Retained readers, audit and reclamation

Deletion cannot silently invalidate independently held live readers or retained
views. Backing remains funded until its actual consumers release it. If a requested
erasure scope conflicts with a retention obligation the caller cannot remove, the
service must refuse that guarantee, not destroy the other consumer's storage.

[Mandatory change capture](change_watches.md) remains in force. Deleting a file
does not erase the audit record of its deletion; change records do not themselves
promise to retain old file bytes. Deleting audit history requires its separate
administrative authority and policy. Storage reclamation and cryptographic erasure
are distinct: neither successful deletion nor key destruction implies immediate
return of physical space.

## Implementation boundary

The [object storage service](../../../source-rs/platform/services/storage/README.md)
has deletion and bounded reclamation, not this full retention-policy interface or
cryptographic guarantee. The current Rust implementation does not provide encrypted
realms or per-reference key destruction. The
[deletion outcome](../../../TASKS_RUST.md#reference-deletion-and-scoped-erasure)
tracks the required service behavior and honest unsupported-policy refusal.
Key hierarchy, persistent encoding and device-specific erasure mechanisms remain
engineering work; raise concrete new semantic/trust choices in
[owner questions](../../../OWNER_QUESTIONS.md) if they arise.
