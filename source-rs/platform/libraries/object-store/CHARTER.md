# Atomic private-catalog lab

This platform library owns bounded catalog operations, transaction validation,
recovery and atomic replacement over `block::Device`. It imports no kernel code.
Start at `Store::transact` in `lib.rs`: validate a root-local change set, encode
an inactive snapshot, flush its body, flush its commit, then publish the result.
`catalog.rs` owns record policy, `protocol.rs` owns bounded connection staging,
`recovery.rs` selects durable snapshots, and `format.rs` owns their encoding.

There are two private roots, four numbered slots per root, and at most 32 bytes
per object. Object numbers 1 through 4 are relative to the service's accepted
connection. A fresh store has an empty object 1 in root 0 for compatibility;
root 1 starts empty. LIST enumerates present numbers. CREATE requires absence;
REPLACE and DELETE require presence. Root revisions increase once per committed
transaction, including delete/recreate; stale expected revisions fail without
writes. There are no paths, ambient lookup, delegation or production realm IDs.

A transaction changes one or two distinct objects in one root. Every change is
validated before any disk effect. BEGIN captures an expected root revision;
staging is private to one connection incarnation. COMMIT consumes the transaction
and rechecks the revision. ABORT discards it. Client replacement discards pending
changes on the next accepted request; provider replacement discards all staging.
Other roots have independent revisions. Reads and LIST are individually coherent;
a sequence of reads is not a snapshot transaction. The demo checks matching root
revisions and app-owned revision tags when loading its related records.

Four 512-byte sectors hold alternating `(body, commit)` pairs. Each body contains
`CTCAT002`, a global commit sequence, then two roots at byte 16 and 184. A root
has an eight-byte revision and four 40-byte entries: presence byte, length byte,
six reserved zero bytes, 32 payload bytes. Unused payload and trailing bytes are
zero. A commit contains `CTCOM002`, matching sequence and body CRC. All integers
are little-endian; sector CRC32 fields occupy bytes 508-511. The sequence orders
whole-catalog recovery; it is separate from the revisions exposed to clients.
Replacement writes the inactive body, flushes, writes its matching commit,
flushes again, then acknowledges. Recovery selects the highest complete pair;
equal valid sequences fail closed. CRC detects accidental corruption and is not
cryptographic integrity or a guarantee against every possible torn write.

The previous `CTOBJ001` format decodes as root 0/object 1. Opening never rewrites
it. The next commit writes v2 into the other pair, leaving the old committed
pair available until the new commit is durable. Later commits may overwrite the
old format; older binaries must not be used to reopen a converted disk. Ordinary
boots preserve existing data images. Unknown nonblank media is never formatted.
Interrupted initial formatting can leave no valid root and fail closed; no repair
tool exists. Once initialized, interrupted replacement retains a complete old or
new catalog. Ambiguous device errors poison the instance until reopened.

Durability assumes flush is honored and previously flushed sectors outside the
write target survive. Malicious media, unrelated sector corruption and lying
caches are outside this model. Tests drop volatile caches and tear every write
and failed flush at each byte prefix, including legacy conversion. QEMU tests
kill the emulator at four write boundaries and after acknowledgements, verify
both app records on reboot, and exercise conversion of actual legacy media.
This is guest power loss, not physical host power-loss certification.

This experimental catalog is not Cathedral's production filesystem DB. It has no
disk allocation, content-addressed blobs, typed schema engine, paths, queries,
snapshots, replication, GC, dynamic realm admission or transactions spanning
kernel authority and storage. Storing both bounded roots in one snapshot means
writes are serialized and share one storage failure domain. The kernel remains
independently usable without this library or its service.
