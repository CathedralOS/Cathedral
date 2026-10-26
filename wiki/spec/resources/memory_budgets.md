# Memory budgets and allocator delegation

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-25 |

## Scope and ownership

The owner ratified hierarchical backing budgets, direct bounded delegation and
optional allocator interposition on 2026-10-25. A matrix may partition its supplied
memory authority among children without forwarding every allocation through its
own process. The kernel enforces grants, conservation, mapping rights and lifetime;
userspace chooses allocation strategy and admission policy. This does not fix
syscall spelling, manifest fields, allocation geometry or a service wire protocol.

Allocation policy may be an ordinary local library or a separate service. A
service may manage a granted pool of physical backing and delegate portions of it;
holding allocation authority need not give it a readable or writable mapping.
Actual access must match the promised custody: a provider cannot retain unrestricted
write access while granting exclusive client access. Hardware separation follows
the [IPC mapping granularity](../communication/ipc.md#shared-regions-and-capacity),
not arbitrary byte-sized subdivisions of one page.

## Conserved hierarchical budgets

A matrix's backing envelope bounds allocations charged to it and its descendants.
Delegation partitions available funding; it does not duplicate it. For example,
an envelope of 256 MiB may assign 128 MiB to one child, 64 MiB to another and
retain 64 MiB. Those amounts are portions of the same envelope, not additional
allowances on top of it. Descendant delegation obeys the same rule.

The accounting distinguishes unused capacity, outstanding allocation reservations,
delegated capacity and retained charges. The same capacity cannot fund concurrent
operations twice. Partial allocation failure returns only the resources actually
released and reservations no longer owed. Changing or retiring an account cannot
erase live obligations or make another child spend them. Exhaustion refuses the
operation without silently borrowing a sibling's grant or expanding an ancestor's
ceiling.

A ceiling alone does not promise physically reserved RAM, successful allocation,
or bounded allocation latency. A provider promising a reserved pool must retain
the actual backing. Other independently enforced limits may still refuse an
operation. Exact sizing, rounding, replenishment and shrinking APIs remain outside
this partial contract.

## Direct grants and optional interception

A process with a direct bounded grant may invoke the corresponding kernel
operations without a request to every ancestor matrix. All such operations check
the caller's actual authority and funding. Importing another library, knowing a
syscall number or issuing its instruction cannot bypass those checks.

A matrix requiring interception instead supplies access to an allocator provider
and withholds direct allocation authority for the mediated resource. The provider
uses its own admitted grants or explicitly delegated funding and passes backing
through authorized handoffs; a request alone cannot charge an unrelated account.
Any alternate direct grant or other provider must be considered when claiming
complete interception. A friendly API wrapper is not an enforcement boundary.

Passing through an intercepting process remains an IPC operation even when it
forwards unchanged. Direct delegation removes that intermediary from subsequent
requests; deciding to forward each request does not. Broker discovery belongs to
connection setup, not every allocation. Separate-process allocation and recursive
matrix forwarding are not mandatory paths.

Ordinary allocators may acquire backing in chunks and allocate objects locally
within it. An external provider can observe backing requests, not every local
object allocation. More detailed instrumentation is additional work, not a property
of page provisioning. A deployment may choose interception for auditing, failure
injection or policy; it must not imply that all applications pay that cost.

## Retention and the accounting boundary

Sharing existing backing does not allocate another copy of its physical pages or
create more funding. Its retained charge cannot be refunded merely because the
original process releases its handle, exits or loses its matrix. Accepted readers,
live mappings, pending grants and unfinished safe reclamation keep their applicable
obligations until discharged. Account retirement must preserve those charges;
closing communication is not revocation of an independently accepted memory grant.

Each induced resource still needs accounting: recipient mappings and page tables,
handles, kernel objects and pending grant records are not free merely because the
payload pages already exist. A backing-only ceiling must not be advertised as a
total resource bound. A total memory bound must also cover attributable metadata
and other induced memory, through explicit charges or bounded funded reservations.
No unbounded kernel allocation may hide behind a bounded payload grant.

The same physical backing is not charged again merely for being mapped by another
process, but that process's mapping and handle resources remain separately bounded.
An independently funded provider may supply already-charged backing; accepting it
does not manufacture capacity in the recipient's allocation envelope. A backing
budget bounds sponsored allocations, not every byte accessible through independently
supplied mappings. Stronger mapped-byte limits are separate admission limits.

This settles conservation and retained charging, not automatic sponsorship transfer,
forced revocation, shared-cache cost splitting or exact refund routing after owner
death. None may reclaim live backing or remove its charge without discharging or
explicitly transferring the corresponding obligation. CPU accounting remains the
separate [executing-program rule](execution_budgets.md).

## Implementation evidence and remaining work

The Rust [physical funding owner](../../../source-rs/kernel/core/user_tasks/resources/physical/README.md)
already describes reserved frames, conserved descendant funding, allocation rollback
and persistent charges after producer death. Its fixed sizing and arena limits are
lab policies, not the production allocator ABI. Kernel heap/IPC reservations remain
separate bounded costs; this is not evidence of a complete total-memory budget.

The [shared-memory contract](../communication/ipc.md) owns grant acceptance and
mapping/custody. The [initial root extent](root_extent.md) is an earlier Omega boot
milestone, not proof of this complete allocator path. The
[implementation outcome](../../../TASKS_RUST.md#memory-budgets-and-allocator-delegation)
owns direct-versus-interposed execution and bypass/conservation acceptance. No
specialized malware-lab or optimizer service is required by this ratification.
