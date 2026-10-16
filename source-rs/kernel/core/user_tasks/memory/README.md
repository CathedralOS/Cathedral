# Sealed objects and local mappings

Start at [memory.rs](memory.rs): traps defer mapping/allocation work to the boot
stack. [Regions](regions/regions.rs) own names, quotas and object lifetime;
[readers](regions/readers.rs) separate read capabilities from virtual mappings.
[Objects](objects.rs) perform freeze/map/unmap and escrow retirement;
[retirement](retirement.rs) removes aliases before refunding physical frames.

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
56 through 61 in [the shared ABI](../../../../contracts/user.rs), not stable OS APIs.

## Funding and limits

Backing remains charged to its original persistent account, including after its
producer exits or releases its handle. Each copied read right charges the
recipient's reader-page quota and one of four reader-capability slots, whether
queued, mapped or unmapped. Multiple rights to the same object are individually
charged. Unmapping returns a virtual slot, not the capability/page reservation.
The current maximum is sixty-four pages per object/account quota; the global arenas
are backed for the twelve-account lab. No extra frames are allocated by forwarding.
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

The integrated [matrix image path](../../../../platform/services/matrix/world/surface/README.md)
uses the same API for retained bitmaps. Matrices clip metadata and keep unmapped
read rights; the final compositor maps the original backing. The app owns the
content and funds its lifetime; process-local pixel storage is not copied at each
ancestor. Software rasterization still reads pixels and writes the final scanout.
