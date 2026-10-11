# Experimental user runtime

Owns x86-64 task entry, syscall wrappers and the static ELF linker layout for the
Rust lab. Depends only on shared `contracts/`; never imports kernel or
distribution implementation. Applications choose their behavior and return an
exit status through `entry!`. The `ipc::Handle` wrappers expose local grants and bounded send/receive/revoke
calls, including deadline receives. `ipc::channel` creates an ordinary one-way
channel; `send_with` moves one dynamic endpoint and `receive_with` explicitly
accepts it. `close` invalidates every local copy of that name. They do not perform
service discovery or choose service policy. The [channel guide](../../../distribution/lab/channels/README.md)
connects these wrappers to kernel enforcement and the ordinary-worker proof. The `task` wrappers expose indexed, bounded launch grants,
child outcome collection, deadline waits, cancellation and explicit connection
acceptance. `Launch::spawn_with` moves up to four dynamic endpoints atomically
with successful admission. `entry_with_environment!` passes their ordered
`Environment` to main; `Environment::load` reads only the caller's original
startup names and never restores closed grants. Protocol roles belong to the app.
The bundle supplements the existing profile-controlled boot/device grants.
`Launch::spawn_tree` also lends idle launch/peer budgets at that admission boundary.
The child enumerates only its assigned grants and may lend them further. Subtree
teardown returns budgets with fresh names to surviving lenders; per-slot executable
and resource ceilings stay fixed. The runtime chooses no manifest or budget policy.
`time` reads the boot-granted monotonic clock; it grants no authority
and chooses no deadline policy. Restart decisions
belong to the caller, not this library. The runtime has no heap allocator or unwinding.
`memory::Private` owns a bounded zeroed page buffer. Sealing consumes mutable
access and returns a `Sealed` token for one authorized peer. `Shared` accepts a
read-only lease and releases it on drop. A producer crash cannot revoke a live
reader's Rust borrow. The `raw` module exposes the same ABI for runtimes and
hostile fixtures; release and sealing require callers to end their own borrows.
Dropping a producer buffer while its peer still holds authority cannot free it;
that abandoned owner reference remains charged until task exit. Normal protocols
must wait for consumer completion and then explicitly release the producer.
`display::mapping` retrieves only the caller's boot-installed framebuffer geometry;
pixel access and drawing requests belong to the separate platform display service.
`keyboard` exposes only the granted raw PS/2 byte channel and bounded writes;
controller configuration and physical-key decoding belong to the input service.
Timed IPC/raw reads require explicit clock authority. Timeout cancels the local
wait; callers choose how to handle remote requests and late replies.

The calling convention and fixed image layout remain experiments, not frozen
Cathedral interfaces. See the [lab guide](../../../README.md) for build commands,
loader restrictions and the remaining admission-policy boundary.

`link` accepts boot-approved client/service connections without task-control
rights. Each incarnation requires explicit acceptance; peer replacement retires
old endpoint tickets. `ipc::wait_two` checks both receive grants, waits without
consuming data, and requires a clock grant. `server` combines a private control
pair with up to two client links, rotates ready channels fairly, and reconnects
replaced clients. Accepted link positions and endpoint incarnations accompany
requests; message contents cannot select these identities. Control-only services
need no clock. Multi-channel idle waits check the third channel within one tick.
Full data reply queues do not block control or other clients. `storage` supplies
synchronous catalog exchanges, with one outstanding request and no automatic
mutation retry. Caller code owns reconciliation and application schemas.
