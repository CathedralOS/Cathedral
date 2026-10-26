# Execution budgets

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-26 |

## Executing program pays

The owner accepted on 2026-10-23 that program execution consumes the executing
program's CPU budget within its enclosing ownership ceilings. Every program's envelope
bounds its owned execution, including descendants; it does not account for every
external effect those programs can request.

Spawning children subdivides the parent's available execution; it does not create
extra budget outside that envelope. Matrix is a program role, not a privileged
scheduling category or an automatic priority boost. Enforcement follows the
ownership hierarchy without prescribing a whole-tree walk at each dispatch.

A service is an ordinary program accepting requests. Neither serving requests nor
being a matrix creates a special scheduling class, extra budget or authority to
charge another program. Work executed by an independently owned provider consumes
that provider's allowance, even when the requesting caller is blocked. Forwarding
a request does not transfer the caller's CPU budget or priority to the recipient.

An external provider may choose to spend its allowance serving a sandbox. This is
authorized use of the provider's resources, not an expansion of the sandbox's own
execution ceiling. The provider must remain within its own enclosing ceilings.
CPU already consumed remains charged when a request is cancelled or its caller
dies; cancellation does not erase usage.

## Owner-controlled subtree suspension

On 2026-10-26 the owner accepted suspension as an execution gate on an owned
program subtree. With the required lifecycle authority, suspending a program
prevents execution of that program and all its owned descendants. The scheduler
enforces the gate without relying on those programs to cooperate. This applies
to every program, not only to programs acting as matrices.

Resume removes only the corresponding suspension gate. Descendant-local and
other ancestor gates still apply, as do existing ceilings and accounting; resume
does not mint a fresh budget or clear debt. The operation must not report the
subtree suspended while its execution can continue. Multicore enforcement and
the syscall representation are implementation work, not a different guarantee.

This is owner-controlled program suspension, not an ordinary task parking on a
notification or an Omega `suspend` call. A waiting parent task does not implicitly
suspend its children. Readiness notifications cannot bypass a subtree gate.

Suspension retains grants, memory and other outstanding custody. It neither
revokes storage access nor destroys encryption keys. Independently owned services
remain schedulable under their own trees and may finish already-submitted work;
using a service does not make it a descendant. Resume does not resurrect a grant
that independently expired or was revoked. Session policy may select suspension
when [locking interactive access](../human_surface/compositor_and_seat.md#session-locking),
but neither operation implies realm locking.

## Admission belongs to the program

Each program chooses what work to accept and how to share its available capacity
among connections. It may use quotas, bounded queues, backpressure, rejection or
its own request scheduling. Accepting excessive work can exhaust its allowance
and harm its responsiveness; it cannot authorize exceeding its enclosing ceiling.
The kernel enforces execution budgets without interpreting application requests
or assigning their causal cost to another program.

This rule does not require identifying one payer for an entire interaction. An
app rendering a large image and a compositor scaling it each spend their own
allowance for their own execution, including the costs of their chosen policies.

## Scope and deferred mechanisms

Caller-funded execution and CPU donation are deferred, not prerequisites for the
shipped coordination service. Any future CPU budget delegation needs a demonstrated
workload and a separately accepted general capability contract available to
ordinary programs. It must not depend on a blessed service classification.

This partial contract does not specify fairness, priority inheritance, hard
real-time guarantees, precise kernel/interrupt cost attribution, or CPU ceiling
enforcement granularity, or metering of storage, network and device quantities.
[Memory budgets](memory_budgets.md) separately settle backing conservation,
retained charging and allocator delegation; they do not imply CPU donation.

## Implementation evidence

The [Rust scheduler](../../../source-rs/kernel/core/scheduler/README.md) charges
measured execution through the executing account's ownership chain and gates
exhausted branches. An independent provider continues to spend its own tree's
budget while handling another tree's requests. This matches the accepted baseline;
the lab's timer granularity, overrun debt and kernel cost attribution remain
experimental, not a production timing guarantee.

Budget-exhaustion gating alone is not evidence for authorized suspend/resume.
The [lifecycle outcome](../../../TASKS_RUST.md#next-work-in-recommended-order)
retains subtree suspension, nested gate composition and session-lock acceptance.

The broader [scheduler design](../../design/part_2_components/01_scheduler_and_resources.md)
retains research on scheduling, physical reservations and other resource families.
