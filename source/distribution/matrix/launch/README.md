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
(`custody.rs`). `mod.rs` is plumbing only. The private native adapter is
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

The read is asynchronous and the service continues handling independent input.
Admission, binding, placement, activation and publication currently call the
existing synchronous Rust mechanisms in the same service turn. Their explicit
completions make the remaining blocking calls visible; this change does not claim
that every launch stage yields or that the matrix event loop is already Omega.

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
