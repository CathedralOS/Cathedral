# Stock matrix request routing

Start at [main.omg](main.omg): authenticated connection class, decoded request,
then the operation to execute. This runs for ordinary F5 stock matrices at every
level, from the session's `Request` operation. The
[Rust dispatcher](../../../../source-rs/distribution/matrix/service/requests/requests.rs)
derives caller facts from the server's accepted connection, decodes the packet,
asks Omega, and executes the selected operation with the original request.
Request words never supply a caller class, incarnation or capability.

The router distinguishes owner control, physical source, child interaction,
storage and separate producer connections. Service acquisition still goes through
the existing grant checks. A storage connection cannot issue input or layout
commands; a child with a separate producer connection cannot inject through its
ordinary interaction connection. Unknown callers fail closed.

Packet shape selects the app workflow; target identity, resource state, geometry
and grant validation remain with that workflow. Malformed app operations still
reach the local world validator instead of being mistaken for a valid launch or
close. Storage routing preserves high scope bits. Sealed storage attachments have
an explicit operation allowlist and require the authenticated storage connection;
refusal drops the attachment without starting a request.

[owner.omg](owner.omg) selects owner diagnostics, the fallback after child control
declines a request, and successful update consequences. No failed operation can
request remembered storage, recovery clearing or surface replay. The host applies
these in the existing order: remember storage, notify local policy owners, clear
recovery, flush/replay, then ordinary invalidation notification before the reply.
Asynchronous child control, launch, close, selection and storage retain their
original pending request; choosing an operation does not manufacture a reply.

[operations.omg](operations.omg) is the private host vocabulary.
[desktop.omg](desktop.omg) binds stock distribution extensions; platform wire
families come from [the matrix reference bindings](../../../platform/contracts/reference/matrix.omg).
Neither is a new service ABI. Omega and Rust execute in one process; this is a
porting boundary, not protection against a hostile host or policy implementation.

The compiled-Omega tests in
[tests/requests.rs](../../../../source-rs/distribution/matrix/tests/requests.rs)
use the Rust protocol constants to check caller separation, malformed packets,
reserved fields, owner success effects, producer separation and every storage
attachment opcode with and without scope bits. Guest verification and current
scope are recorded in `TASKS_RUST.md`.

The next descent remains in Rust: `service/children/children.rs` selects owner
child-control operations, `service/children/launch/request.rs` chooses authorized
launch targets, and request handlers validate and mutate local world state.
Those are further Omega work; this migration does not claim a fully Omega matrix.
