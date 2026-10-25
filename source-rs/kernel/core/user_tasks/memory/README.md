# Page objects and local mappings

Start at [memory.rs](memory.rs): traps defer mapping/allocation work to the boot
stack. [Regions](regions/regions.rs) own names, quotas and object lifetime;
[readers](regions/readers.rs) separate read capabilities from virtual mappings.
[Objects](objects.rs) perform freeze/map/unmap;
[retirement](retirement.rs) removes aliases before refunding physical frames.
[References](references.rs) settle surrendered executable references;
[escrow](escrow.rs) retains custody of discarded IPC attachments.

`SHARE_MUTABLE` permanently distinguishes live single-writer backing from frozen
objects. The producer retains RW/NX; typed shared-view transfers install only
RO/NX rights and cannot be received as immutable images. Both modes reuse the
same funded lifetime/escrow arena. Map checks the mode before returning even an
existing mapping; freeze and executable admission reject mutable backing.
Follow the [runtime and mailbox](../../../../platform/libraries/user-runtime/memory/shared/README.md)
for publication and hostile-writer rules. This slice does not add multi-writer
authority or replace ordinary service transport.

[Allocation](allocation.rs) reserves funding once and zeros at most eight pages
per boot-stack step. The caller stays blocked and the region unmapped until all
pages are ready. Cancellation refunds the unfinished reservation and backing
before slot reuse. Follow the [deferred-work entrance](../execution/deferred/README.md)
for selection, interrupt windows and the remaining non-preemptible operations.

Backing retirement returns at most eight frames per path visit. Explicit release
removes the caller's alias first and keeps it blocked until any final backing
refund completes. Death inherits a pending release; allocation refusal/death
refunds the unbacked reservation once and retires initialized pages in batches.
The retiring region keeps its metadata slot and whole-object logical quota
until completion, while physical funding returns to its sponsor batch by batch.
Other tasks can continue memory operations between visits.

Executable-reference settlement rotates among pending objects. IPC discard takes
a finite snapshot and advances one object at a time; later discards wait in IPC.
Both sets retain their unpublished reader slots through receiver death, preventing
a stale discard bit from consuming a newly reused slot. A consumed slot can be
reused while its old object's backing finishes retirement. These are separate
paths, each with an eight-frame allowance, not an eight-frame total per boot turn.
Mapping removal remains atomic and precedes every backing refund.

## Transfer contract

`Private::freeze` consumes writable access and returns an immutable `Object`.
`Handle::send_memory` sends up to 64 metadata bytes plus a copied read capability
on an ordinary channel. The sender keeps its right. The receiver gains a fresh,
caller-local handle, initially unmapped, through `receive_memory` or
`try_receive_memory`. Holding and forwarding this handle never maps or copies
its backing. `Object::map` installs RO/NX pages; `unmap` removes only that local
mapping; dropping the object releases the capability. Neither unmap nor sender
death withdraws another holder's authority. Only the last reference frees backing.

[The syscall transaction](../syscalls/ipc/capabilities.rs) checks the full user
packet, endpoint and vacant queue before reserving recipient capacity. Commit is
then infallible under the single-CPU kernel lockstep. The queue holds an escrow
reference, inaccessible to the receiver until successful checked copyout.
Wrong attachment types and bad destinations preserve the queued message.
[IPC teardown](../../ipc/lifecycle.rs) reports discarded escrow to the deferred
memory owner, including discard through an escrowed endpoint chain.

The wire packet is the endpoint-transfer layout: length and local handle as two
little-endian u64 words, followed by 64 bytes. Memory and endpoint attachments
use distinct receive operations; plain messages work with either. A raw numeric
handle in message bytes conveys no authority. These are experimental syscalls
56 through 61 in [the shared ABI](../../../contracts/user.rs), not stable OS APIs.

## Funding and limits

Backing remains charged to its original persistent account, including after its
producer exits or releases its handle. Each copied read right charges the
recipient's reader-page quota and a reader-capability slot, whether
queued, mapped or unmapped. Multiple rights to the same object are individually
charged. Unmapping returns a virtual slot, not the capability/page reservation.
An object holds up to 128 pages; private and reader budgets can fund several
objects, since a matrix hub pins images and lends budget downward. Current
per-task and global arena bounds live in the [memory contract](../../../contracts/memory.rs).
No extra frames are allocated by forwarding.
A retained reader can prevent returning or reassigning its sponsor's funding.

Copying read authority permits further forwarding. This does not implement
revocable reads, writable sharing, ownership transfer, or sponsorship migration.
Existing boot-link `seal`/`accept_from` remains a compatibility path for older
storage/rendering fixtures, with its original single-reader completion rule.

## Evidence and consumers

[Host transaction tests](regions/transfer_test.rs) cover lifetime, sponsor charges,
recipient quota exhaustion, unpublished handles, independent mappings, type
mismatch and nested queue discard. The shared [boot proof](../../../boot/uefi/lab/shared_memory.rs)
runs [three ordinary programs](../../../../distribution/lab/shared-memory/main.rs)
on both x86-64 and ARM64. A produces one page, B forwards without mapping, and C
maps after both A and B have exited. Repeated map/unmap, invalid copyout, queued
discard, reader write faults and final frame/heap reclamation are checked.

The [retirement suite](retirement_tests/retirement_tests.rs) checks maximum-size
release, death during release, competing allocation, two pending discard batches,
receiver death and reader-slot reuse against real frames on both ISAs. Every
cleanup visit checks its frame allowance and physical funding conservation.
The [allocation sweep](rollback.rs) refuses every frame allowance and cancels at
each partial batch; [reference retirement](references_test.rs) drains 140
references across 70 objects while preserving a live owner's sealed mapping.
The allocation sweep also seals/unmaps maximum-size aliases in every guarded
slot, including slots spanning leaf tables. [ISA range edits](../../../arch/README.md)
walk each leaf table once; translation writes and the core ledger update remain
atomic, with backing retained throughout. This changes neither capability
publication nor the ordering of final refunds.
Runtime mapping installation validates the complete range before writing any
leaf, then publishes the core ledger only on success. The shared
[mapping refusal proof](../address_space/regions_test.rs) sweeps every possible
collision across a maximum-size range spanning two tables and rejects invalid
backing, checking successful retry and unchanged ledger state on both CPUs.

The integrated [matrix image path](../../../../platform/libraries/matrix/world/surface/README.md)
uses the same API for retained bitmaps. Matrices clip metadata and keep unmapped
read rights; the final compositor maps the original backing. The app owns the
content and funds its lifetime; process-local pixel storage is not copied at each
ancestor. Software rasterization still reads pixels and writes the final scanout.
