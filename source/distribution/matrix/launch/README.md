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
`binding.rs` beside that entrance; `scene.rs` owns visual completion and restoration.
`mod.rs` is plumbing only. The private native adapter is
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
run before the launch workflow can admit a ready image. During a visual transaction,
observing an authenticated owner request marks cancellation before deferring its
mutation. Neither path promises instantaneous cancellation on request submission.
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
input and present over the independent interaction stream during storage waits.
During a staged visual transaction, input still progresses but visual commits
serialize until that scene commits or aborts. The launch can progress past these
held requests, while ready owner mutations still take priority. Stale queued requests, attachments and
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

Ordinary activation now yields through
[scene/scene.rs](../../../../source-rs/distribution/matrix/service/parent/scene/scene.rs).
It prepares bounded clipped packets without changing live visibility, demand,
input routes or child content. Cached image readers stay in matrix custody while
the transaction owns visual geometry. Begin, updates, commit and abort each yield;
tagged input uses the same safe reply dispatcher as child presentations. The
launch owner rechecks custody before COMMIT. Only an accepted commit installs the
new local selection, visibility and management target. Unknown transport outcome
or failed abort fails closed; lateness alone retains the outstanding exchange.

An observed owner mutation can cancel the pending scene while its actual mutation
waits for visual custody. Before COMMIT, cancellation aborts the staged scene.
After COMMIT was sent, acceptance cannot be undone: rollback restores the previous
scene, if the exact previous task still occupies that slot and the owner has not
selected elsewhere, before reclaiming the exact admitted task. Restoration also
handles an owner that already stopped that task, but never touches a replacement
task in its slot. A cancellation
already known at completion does not change local selection/input routes first.
Restoration failure is fatal; there is no false successful cleanup. This
compensating path can temporarily expose the accepted scene before restoration.

Publication is now a local handoff. Admitted-child requests remain held until
Finish, so the child cannot yet own drawn surfaces; there is nothing to flush or
replay for it. Publication notifies the matrix's children and records login state;
Finish releases startup and returns the launch receipt. Kernel admission,
retiring/reclaiming tasks, selecting an already-running app, compensation and some
startup/recovery helpers still contain synchronous work. No claim is made of
concurrent pixel commits or a fully nonblocking matrix.

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

`python tools/boot-harness-rs/desktop_test.py --scene-async MODE --timeout 120`
checks `complete`, `revoke`, `stop`, `failure`, `late-revoke` and `late-stop-target`
on fresh/cold nested desktops. A three-second gate after staging proves fresh input reaches the matrix
before release; Notes must contain that input afterward (its own synchronous
render may wait). Cancellation/failure aborts without publication, reclaims the
admitted task, preserves the previous draft and permits same-slot retry where
the launcher survives. LOSS must promptly report BUSY. The two late modes hold an
already accepted commit reply and prove restoration after revocation or owner
reclamation of the admitted task. All
modes protect close, save/cold-reopen the draft and reclaim the subtree. These
features are absent from normal F5.
