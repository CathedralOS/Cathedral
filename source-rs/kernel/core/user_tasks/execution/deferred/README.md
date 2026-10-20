# Deferred kernel work

Start at [deferred.rs](deferred.rs): advance reclamation, advance at most one
child admission when cleanup is complete, then at most one memory call.
[The boot loop](../execution.rs) provides the suspension boundary after that
step returns. [Traps](../traps.rs) only route to boot; they never use the frame
bank or construct an address space on an interrupt stack.

## Pending work and lifetime

[Pending](pending.rs) contains fixed bitmaps for admissions, memory calls,
cancellations, retired tasks, abandoned drafts, collection and launch returns. Successful syscall acceptance
enqueues the corresponding task/launch. Close removes its memory call and queued
admissions before reuse; it queues physical retirement and cancellation of live
children. Cancellation drains newly added descendants before the first reclamation pause.
Collection and loan return wait until all physical retirement steps finish. Cached child-to-launch routes are fixed by composition, including
reserved dynamic slots; lending changes owners, not those child identities.
Admission rechecks peer consent after queueing: if that incarnation died during
another step, the blocked owner receives `PEER_CLOSED` before image construction
or endpoint publication. Restarting the peer slot does not revive old consent.

Admission and memory selection rotate across set bits, so a low-numbered task
cannot monopolize steps by requeueing. Physical retirement rotates too; withdrawal and final collection preserve
increasing slot order. Queues are selection indexes; capability and lifecycle models remain
authoritative. The task arena is still limited to 31 slots.

Trap-side pending detection checks maintained flags/sets, without scanning tasks,
launches or retired memory words. Escrow retirement and deferred executable
reference release iterate set bits within their bounded word arrays. Expensive
whole-session conservation assertions run during actual reclamation, not every
ordinary return to the boot loop.

## Consistent suspension

The step returns with no exclusive session borrow, unpublished stack-local
transaction, or allocator lock outstanding. If work remains, the ISA provides a
short maskable-interrupt window before the boot loop explicitly yields to an
eligible user. An asynchronous IRQ in the boot context returns there while work
remains. An explicit boot yield may dispatch a user; that user's next trap returns
to boot for another step. Thus runnable peers cannot indefinitely skip deferred
work, and pending work need not drain completely before any peer runs.

The interrupt window returns masked and does not promise that every pending
interrupt was serviced. It also avoids sleeping for an entire timer tick merely
to continue work when every user is blocked. The pinned session is accessed
through the existing raw-pointer suspension contract; the frame bank stays
exclusively on the boot stack, outside trap callbacks.

## Incremental private allocation

[Allocation](../../memory/allocation.rs) reserves quota, physical funding and a
private region once, then acquires/zeros at most eight pages per step. It retains
progress in session-owned state and requeues the blocked caller. No user mapping
is installed and no handle returned until every page is initialized. The caller
cannot run during staging, and other tasks have no authority over that region.

Physical funding counts both allocated frames and the unbacked reservation;
conservation assertions include both. Each batch is charged to the requester.
On failure or caller death, cleanup refunds the unbacked part and releases the
allocated part exactly once. A dead caller's staging is settled before its
address space, funding or slot can be reused. Final mapping still commits as one
operation, with complete rollback on failure.

## Incremental executable admission

[Spawn](../../lifecycle/spawn.rs) revalidates the owner, original peer consent and
initial endpoint bundle, then advances a private [construction](../../admission/construction.rs).
The first step decodes the executable through [image.rs](../../admission/image.rs)
and allocates an inactive root. Supervisor entry mappings are then prepared in
batches: x86 copies at most 64 kernel image/heap/stack leaves per step, preserving
their boot permissions. ARM64 already borrows its permanent supervisor branch
and completes this phase immediately. The ISA retains owned range/cursor values,
never a page-table borrow across suspension. The x86 root accessor refuses an
unfinished supervisor tree. Later steps copy, zero and map at most eight image
or stack pages, then reserve runtime mapping tables and map any approved
framebuffer in separate phases. The generic page cursor knows segments, not ELF.
Initial boot admission remains synchronous because no users are runnable yet.
It runs the same mapping engine to completion before loading the user image.
As with image batching, extra yields can lengthen admission under contention;
they provide other tasks an opportunity to run, not a faster-launch guarantee.

The launch retains the immutable executable source across steps; a draft owns
its segment plan, page cursor, root and frame ledger without retaining borrowed
source pointers. Frames are charged to the child's funding as acquired; each
step's CPU work is charged to its owner. No task root, runnable context or moved
endpoint is published until the final serialized commit. Delegation/grant plans
and the epoch are prepared after the last yield. A queued message can make an
initial receive right unmovable while loading; refusal leaves the message and
right with the owner and rolls back the draft.

Failure and owner/peer death release every private frame exactly once. Owner
return aborts drafts before returning launch capacity to a surviving ancestor.
When no lender survives, it also drops the launch's executable source reference;
otherwise that reference remains available to the surviving owner. Conservation
assertions count both published roots and unfinished drafts.

## Incremental subtree reclamation

[Lifecycle](../../lifecycle/lifecycle.rs) withdraws authority, advances one cleanup
step, then checks conservation. [Reclamation](../../lifecycle/reclamation.rs)
stops every newly canceled descendant before yielding: none may run with a dead
owner. Endpoints, waits and device routes close immediately. This authority pass
still handles the whole bounded task arena in one operation.

Physical retirement rotates between dead tasks. [Alias cleanup](../../memory/closing.rs)
scans at most 32 reader/region entries and removes at most one page object's
mappings/references per step. Live readers retain backing after producer death.
Dropping a dead reader's last reference reclaims backing before another task can
reuse the object index. Unfinished private allocation cancellation and each
object's unmap/free remain whole operations, bounded by their page caps.
After aliases disappear, at most eight private image/stack/table frames are
returned per step, in reverse allocation order. Abandoned admission drafts also
return at most eight frames per step. Every freed frame refunds its owning
account; retirement CPU work is charged to the dead task, draft cleanup to its
requester. No partly retired root can execute or serve checked user copies.

Dead slots remain `Exited` and child outcomes stay unpublished until the cleanup
wave finishes. A final metadata commit collects tasks, returns launch loans and
completes cancellation callers. New admission waits throughout the wave, so
capacity cannot be recycled early and newly launched tasks cannot keep extending
the same wave indefinitely. Other users and their memory/IPC work can continue.
The boot loop drains pending work even when the scheduler has no live users.
Final collection and whole-session conservation assertions are still atomic.

## Limits and evidence

This is not a bounded-time kernel: executable decoding, bare-root allocation, runtime
table reservation, framebuffer mapping, final authority commit, subtree
authority withdrawal, final collection, failure rollback, final page-object mapping,
sealing/unmapping, heap operations and event bursts still run uninterrupted.
Subtree physical retirement is now staged, but single-object cleanup and pending
reference settlement still need smaller work allowances before latency claims.
Eight pages is an engineering work allowance, not a measured or certified latency limit. No new priority,
reservation or application ABI is introduced.

Smoke builds buffer phase observations in [execution timing](../timing.rs),
reporting only after a session stops. The [timing audit](../../../scheduler/timing.md)
describes the competing-memory workload, report command and measurement limits.
Page-object seal/unmap use one ledger traversal; they remain atomic and preserve
all unrelated mappings before refunding any backing.

[Host tests](tests.rs) cover queue coalescing, fair requeueing, cancellation and
reuse, dynamically added descendants, and boot/user routing. The real-frame
[rollback fixture](../../memory/rollback.rs) fails allocation at every frame
boundary and cancels at every partial batch of a maximum-size object, checking
absence of mappings and exact funding/frame refunds. The
[admission fixture](../../admission/rollback.rs) builds a multi-batch image,
fails at every physical-frame allowance, cancels at every phase boundary and
checks unpublished task/IPC state, zeroed BSS, final permissions and exact refunds.
It also exercises owner/peer death, newly busy initial rights and abandonment of
an object-backed executable through the production lifecycle on both ISAs. A
committed parent/child teardown checks the eight-frame allowance on every step,
endpoint withdrawal before the first pause, an unaffected ready peer, unavailable
slots/outcomes throughout cleanup and exact final refunds.
Existing smoke and desktop fixtures exercise IPC, deadlines, repeated task
replacement and subtree teardown.
