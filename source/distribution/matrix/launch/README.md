# Stored-program launch workflow

Start at [main.omg](main.omg). The ordinary F5 path for launching Notes, Colors,
a user workspace or a nested matrix executes this workflow. Omega issues read,
retire-picker, admit, bind-storage, place, activate, publish and finish commands.
Only applicable steps run: an owner-directed/headless launch needs no child
placement, an app without storage skips binding, and desktop activation belongs
to the stock desktop role. Failed work reclaims an admitted task and schedules
picker recovery if the old picker was deliberately retired.

[The Rust entrance](../../../../source-rs/distribution/matrix/service/children/launch/launch.rs)
dispatches these commands. Its siblings own caller validation (`request.rs`),
resource mechanisms (`steps.rs`) and identity/cancellation/reply custody
(`custody.rs`). Asynchronous storage completion and grant commit live in
`binding.rs` beside that entrance. `mod.rs` is plumbing only. The private native adapter is
[policy/launch.rs](../../../../source-rs/distribution/matrix/policy/launch.rs).

A matrix issues a non-reused operation ticket shared with launcher recovery.
Each command is emitted once and must complete with both its ticket and command.
A duplicate, wrong-stage or old-operation completion cannot advance work. Rust
holds the authenticated request, exact caller task/connection identity, launch
plan and sealed image. Those values never become Omega-granted authority.
Cancellation discards the image; rollback targets only the task this operation
actually admitted. Completion never replies through, disconnects, or records a
receipt for a replacement caller connection.

The host rechecks caller liveness, storage scope and any already-known parent
storage identity before consequential steps. Deliberately retiring the login
picker is the exception: its accepted selection continues under the matrix's
custody. A processed owner change to either the caller's or target's relevant
authority cancels pending work, including STOP of an already-empty target.
The existing storage exchange still drains its response. Deferred owner requests
run before the launch workflow can admit a ready image; cancellation is effective
when the owner request is processed, not instantaneous on request submission.
A failure during task teardown is fatal to the matrix rather than acknowledged
as successful cleanup; kernel subtree ownership remains the final reclamation
boundary. Recoverable launch refusals leave siblings running.

The private bridge uses operation 11. Events 0..3 initialize, issue a command,
complete a command and cancel. State words 16..21 retain phase, ticket, role flags,
error and task/picker obligations. Facts 2..5 carry the ticket/command/status and
whether admission supplied storage. No endpoint, task ID or object enters this
packet. This is distribution implementation detail, not a public app ABI.

Ordinary program reads and storage binding now yield to the service loop. The
[storage binding exchange](../../../../source-rs/distribution/matrix/service/parent/storage/binding.rs)
owns the storage reply stream until completion or retirement. Its refresh result
is retained without granting authority until queued owner mutations run and the
launch owner rechecks the exact task, caller and scope. A completion may be held
only while Omega still expects that ticket and command (private event 4).

The newly admitted task's requests wait until publication or rollback, so its
startup broker cannot observe a half-configured environment. This hold affects
only that task; loss surveys remain BUSY throughout the pending launch, including
the interval between reply arrival and grant commit. Other apps still receive
input and present over the independent
interaction stream. The launch can progress past these held requests, while ready
owner mutations still take priority. Stale queued requests, attachments and
replacement-connection disconnects are discarded rather than applied to a reused
slot. Local storage revocation no longer refreshes upstream: withdrawing a grant
or reclaiming a task does not depend on the service being withdrawn. Future
forwarded operations still validate their stamped upstream binding.

The production binding deadline remains 100 ticks, as in the synchronous path.
Malformed replies, channel failure or timeout retire the untagged storage stream
so a late reply cannot answer subsequent work. A separate storage-stream failure
leaves input and existing processes alive, but their storage is unavailable until
re-established; an unsaved draft is not thereby persisted. Legacy fixtures sharing
one interaction/storage stream retain fatal failure handling. Existing manual
owner grants, recovery and startup paths still use the synchronous grant helper.

Admission, placement, activation and publication remain synchronous host steps.
Activation/publication currently share the upstream request/reply channel with
input polling. `upstream_busy()` excludes polling while presentation owns it;
changing only the waits would not let new input through. The next step must
separate input/presentation transport or safely dispatch concurrent replies, and
preserve atomic scene commit/abort and cancellation before migrating these steps.
This is an implementation frontier, not a new owner semantic decision.

## Verification

The Rust host tests execute compiled Omega for all role/storage combinations,
wrong/duplicate completions, replacement tickets, cancellation and failure at
every ordinary stage. They verify rollback and picker-recovery obligations.

`python tools/boot-harness-rs/desktop_test.py --launch-async MODE --timeout 240`
runs the full F5 composition with a nested user desktop. Modes are `complete`,
`revoke`, `stop`, `bind` and `place`. A three-second read gate must leave sibling
Notes accepting input. Real routed owner revocation or launcher stop prevents
admission. Binding/placement fault modes fail after task creation and require
reclamation before a successful retry in the same slot. Every mode protects,
saves and cold-reopens the sibling draft and finally closes the nested subtree.
The gate, owner probes and failure injections are disposable profile features;
normal F5 enables none of them.

`python tools/boot-harness-rs/desktop_test.py --binding-async MODE --timeout 240`
checks `complete`, `revoke`, `stop` and `timeout` on fresh/cold nested desktops.
Sibling Notes must visibly consume a key before the binding gate releases. Real
owner cancellation must prevent grant commit and publication, reclaim the admitted
task and preserve the draft; revocation then permits a fresh successful launch.
The timeout fixture drops one real reply, then exercises the ordinary empty-channel
deadline and stream retirement. It preserves the live draft and close protection,
then explicitly discards it for subtree cleanup; cold reopen stays empty. It does
not claim automatic storage reconnection or a successful save after retirement.
