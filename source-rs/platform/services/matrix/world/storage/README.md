# Matrix storage delegation

Start at [storage.rs](storage.rs) for exclusive source assignment, connection
identity and local binding revisions. [The parent adapter](../../service/parent/storage.rs)
checks that policy and forwards the [storage protocol](../../../../../contracts/storage.rs)
through the same matrix executable at every level. The root adapter alone holds
the storage provider's links. Kernel IPC transports these requests; the kernel
does not implement catalog operations or namespace policy.

The lab root mode `matrix::STORAGE_ROOT` reserves its first two supplied links for
the provider's two private catalogs. Owner-only `STORAGE_GRANT` assigns one source
to a live child connection. Each source can belong to only one direct child;
the nested matrix can delegate its one inherited source to one of its children.
This delegates whole roots, without creating or subdividing directories.
The root can combine these links with the four-endpoint display/input environment.
The visible desktop uses that composition; the dedicated storage fixture exercises
the same delegation without a display adapter.

A child obtains only its local binding revision through `STORAGE_INFO`. Each data
opcode carries that revision in its upper 48 bits and `STORAGE_BASE + operation`
in its lower 16 bits. Other request fields and replies retain `storage.rs`'s
format. Every matrix authenticates the connection and revision, then substitutes
its own upstream revision. No child can select a provider root, sibling, ancestor
or global process. Keeping roots exclusive between siblings also keeps root
generations and transaction state independent. Binding revisions do not measure
sibling activity.

Revocation aborts the upstream transaction before acknowledgement. Child exit,
fault or cancellation uses the same path. Changing an inherited binding clears
descendant grants; regranting the ancestor never implicitly revives descendants.
Each request checks its revision at every hop, including requests already in
flight when authority changes. Connection replacement requires explicit grants.
Matrix or provider transport failure retires the matrix rather than guessing
whether an uncertain mutation committed. Recovery reconciles committed data
through fresh connections; there is no automatic mutation retry.

Run `python tools/boot-harness-rs/run.py --matrix-storage-test --release` from
the repository root. [The fixture](../../../../../distribution/lab/matrix/storage/storage.rs)
checks sibling data/list/generation isolation, over-delegation, stale requests,
ancestor revocation, abandoned transactions, replacement, and committed data
after reopening the provider and matrix tree. It uses a private reset test disk.
Host tests cover exclusivity, stale incarnations and binding revisions.

This bounded experiment retains the provider's two roots, four objects per root
and 32-byte records. Read-only attenuation, namespace creation, shared writable
catalogs and storage quotas remain future work. The visible desktop saves private
app colors through this boundary and verifies them after a cold reboot. Parents remain trusted to delegate their own authority; this is not a
claim of timing isolation or confinement from a malicious ancestor.
