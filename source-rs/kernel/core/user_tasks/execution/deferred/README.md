# Deferred kernel work

Start at [deferred.rs](deferred.rs): reclaim retired tasks and withdrawn launch
authority, admit at most one child, then advance at most one memory call.
[The boot loop](../execution.rs) provides the suspension boundary after that
step returns. [Traps](../traps.rs) only route to boot; they never use the frame
bank or construct an address space on an interrupt stack.

## Pending work and lifetime

[Pending](pending.rs) contains fixed bitmaps for admissions, memory calls,
cancellations, retired tasks and launch returns. Successful syscall acceptance
enqueues the corresponding task/launch. Close removes its memory call and queued
admissions before reuse; it queues physical retirement and cancellation of live
children. Cancellation drains newly added descendants before reclamation and
loan return. Cached child-to-launch routes are fixed by composition, including
reserved dynamic slots; lending changes owners, not those child identities.
Admission rechecks peer consent after queueing: if that incarnation died during
another step, the blocked owner receives `PEER_CLOSED` before image construction
or endpoint publication. Restarting the peer slot does not revive old consent.

Admission and memory selection rotate across set bits, so a low-numbered task
cannot monopolize steps by requeueing. Cancellation/reaping preserve increasing
slot order. Queues are selection indexes; capability and lifecycle models remain
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

## First incremental operation

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

## Limits and evidence

This is not a bounded-time kernel: ELF admission, subtree reclamation, allocation
rollback, final mapping, sealing/unmapping, heap operations and event bursts can
still run uninterrupted. Subtree authority withdrawal and loan return have not
been split across user execution. Those need explicit progress/cancellation
state before another suspension boundary is safe. Eight pages is an engineering
work allowance, not a measured or certified latency limit. No new priority,
reservation or application ABI is introduced.

[Host tests](tests.rs) cover queue coalescing, fair requeueing, cancellation and
reuse, dynamically added descendants, and boot/user routing. The real-frame
[rollback fixture](../../memory/rollback.rs) fails allocation at every frame
boundary and cancels at every partial batch of a maximum-size object, checking
absence of mappings and exact funding/frame refunds. Existing smoke and desktop
fixtures exercise IPC, deadlines, repeated task replacement and subtree teardown.
