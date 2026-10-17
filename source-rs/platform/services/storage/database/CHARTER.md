# Experimental database storage provider

Start at [main.rs](main.rs): fund the heap, open the assigned volume, then
[serve](service.rs) the bounded private-object protocol. The
[wire definition](../../../contracts/database.rs) imports no implementation.
The [publication library](../../../libraries/database-objects/lib.rs) owns atomic
records; the [engine adapter](../../../libraries/storage-volume/engine/lib.rs)
owns redb's bounded block region. The kernel grants memory, IPC and device access;
it contains no content representation or database policy.

Each data connection is bound to one provider-local root. Neither message bytes,
PIDs nor content digests can select another root. This two-root mapping is explicit
composition policy, not discovery or authentication by caller-supplied names.
The supervisor connection can close the provider but cannot read/write roots.

PUBLISH validates the request and attachment sizes before mapping immutable input.
Preparation hashes bounded contents before acquiring the writer; the writer
rechecks the expected revision. Chunks, manifest, head, event and retained-byte
charge commit together with immediate durability. A quota failure aborts all
staged records. Engine/corruption errors retire the provider rather than returning
a clean refusal or continuing on poisoned storage. An ambiguous/lost reply requires
head reconciliation; there is no request-ID replay cache.

No mutable client buffer is hashed and later reread. Attachments are kernel-frozen
objects, received read-only, dropped before replying. One request is handled at a
time; bounded kernel queues and reader-page funding bound outstanding attachments.
The fixed heap has no hidden growth. Diagnostic samples live in [metrics.rs](metrics.rs).

READ validates the head's event and manifest, then the requested chunks. It requires
an expected revision so multiple range requests cannot silently mix versions.
HEAD reports current metadata, not a full scrub of every retained payload. Content
digests authenticate bytes without conferring lookup authority. Retention is bounded
by a logical record quota; physical engine overhead and maintenance headroom are
separate concerns. A clean capacity status currently means this logical quota only.

The distribution's [lab guide](../../../../distribution/lab/objects/README.md)
owns composition, commands, evidence and limitations. This provider is a separate
binary; the ordinary catalog's format and matrix delegation remain unchanged.
