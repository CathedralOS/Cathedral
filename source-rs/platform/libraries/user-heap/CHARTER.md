# Explicitly funded userspace heap

[lib.rs](lib.rs) owns a locked first-fit allocator over up to four exclusively
supplied, pinned extents, each one funded page object; allocation prefers earlier
extents and a free returns to the extent that holds it. It does not issue
syscalls, request growth, or own permission policy.
[user-runtime/heap.rs](../user-runtime/heap.rs) obtains as many private-page
objects as the per-object cap requires and installs them for the process
lifetime. The kernel reclaims them on exit or fault; individual frees make space
reusable inside the grant rather than refunding pages.

Use the runtime's optional `heap` feature, declare a global `Heap::empty()`, then
initialize it before allocating. Fallible collection APIs can report exhaustion;
infallible allocations can terminate the process. This is not a universal promise
that all third-party libraries recover from OOM. There is no unwinding.

Stats count requested live/peak bytes, largest allocation and refusals. They exclude
alignment, free-list overhead and fragmentation. Three host tests cover
initialization, alignment, exhaustion, coalescing/reuse, preservation of live
data, and overflow into further extents with frees returning home. The
[guest database proof](../../../distribution/lab/database/README.md) checks funded
installation, quota refusal and reclamation across normal exit and a contained fault.
