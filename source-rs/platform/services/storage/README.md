# Object storage provider

A separate user task owns the exclusive boot-granted disk controller and the
[object store](../../libraries/database-objects/README.md). [main.rs](main.rs)
funds the heap, opens the guarded volume and recovers the engine, then
[service.rs](service.rs) accepts connections and dispatches the
[object protocol](../../contracts/storage.rs): one 64-byte packet each way, a
sealed attachment for publications and, through [bulk.rs](bulk.rs), a sealed
attachment for bulk reads. The private control connection answers health and
the supervisor's close; it conveys no object access.

Two boot-approved data links bind to private roots 0 and 1 by their
service-local link position. The kernel isolates principals and grants
endpoints; the provider enforces root-local object access, and no message
carries a root selector. Within a root, objects form a tree, and a request's
three scope words name subtrees it must stay inside: before dispatch the
provider checks that every stamped scope is an ancestor of every object the
request would touch, refusing an object outside the scope and one that does
not exist alike. The desktop's root matrix holds both links and delegates
subtrees of them to its children; the [objects lab](../../../distribution/lab/objects/README.md)
hands them to two bare clients instead.

Every write is one engine transaction. Collection runs before each publication
or copy, including a subtree copy, so retired history never blocks a live
root; a refused write leaves no partial record. An engine or corruption failure ends the provider rather than
serving uncertain state, and the distribution replaces it and regrants roots.

The `recovery-lab` feature adds the owner-controlled [fault gate](gate.rs),
which stalls one root's next matching request before execution or after
commit, so proofs can exercise lost acknowledgements and provider replacement.
`trace-storage` emits per-request timing and ATA counters for the
[timing summary](../../../../tools/boot-harness-rs/storage_timing.py). Ordinary
builds contain neither.

The provider's reader pages receive publication attachments; its private pages
hold the heap and the pages behind bulk replies, which stay charged to it until
the last reader releases them. Profiles fund both explicitly.

Proofs, from the repository root:

```powershell
python tools/boot-harness-rs/objects_test.py --release
python tools/boot-harness-rs/objects_test.py --retention --release
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/matrix_database_test.py --release --recovery
```
