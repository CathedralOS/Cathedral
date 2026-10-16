# Explicitly funded userspace heap

[lib.rs](lib.rs) owns a locked first-fit allocator over one exclusively supplied,
pinned extent. It does not issue syscalls, request growth, or own permission policy.
[user-runtime/heap.rs](../user-runtime/heap.rs) obtains a private-page grant and
installs it for the process lifetime. The kernel reclaims it on exit or fault;
individual frees make space reusable inside the grant rather than refunding pages.

Use the runtime's optional `heap` feature, declare a global `Heap::empty()`, then
initialize it before allocating. Fallible collection APIs can report exhaustion;
infallible allocations can terminate the process. This is not a universal promise
that all third-party libraries recover from OOM. There is no unwinding.

Stats count requested live/peak bytes, largest allocation and refusals. They exclude
alignment, free-list overhead and fragmentation. Two host tests cover initialization,
alignment, exhaustion, coalescing/reuse and preservation of live data. The
[guest database proof](../../../distribution/lab/database/README.md) checks funded
installation, quota refusal and reclamation across normal exit and a contained fault.
