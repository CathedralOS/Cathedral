# Physical-memory funding

Start at [the funding model](mod.rs). It conserves reservations between
persistent accounts and tracks occupied frames. [The frame bank](../../memory/frames.rs)
reserves actual physical frames before any process starts; it rolls back a partial
bank reservation and returns the whole bank after session teardown. Idle bank
frames are unavailable to other allocator users. This is bounded, eager backing,
not a promise based on observing global free memory.

## Admission and ownership

Bootstrap currently gives each bound account 256 frames plus its private-page
quota. Each dynamic process slot contributes another 256 frames to the initial
process's allowance. Dynamic accounts start unfunded. This is a lab default,
not a manifest or production memory-sizing decision.

`Executable::reserve_funded(limits, child_slots, admission_frames, physical_frames)`
atomically subdivides physical funding with process/page/IPC reservations. The
short `reserve` client requests 256 frames per selected account and descendant
slot. The raw `TASK_RESERVE` record is six u64 words: private pages, reader pages,
channels, descendant slots, per-load frame ceiling, physical frame reservation.
A load ceiling remains a separate failure bound; it does not create funding.

`Launch::assign_memory` sets an owned idle account's reservation, returning any
reduction to its owner. `resources::physical_budget()` reports only the caller's
reserved and charged frames, never global free memory or another account's use.
The kernel syscalls are `PHYSICAL_ASSIGN` (55) and `PHYSICAL_BUDGET` (54).

[Admission](../../admission/mod.rs) limits every allocation to the
account's unused funding and the per-load ceiling. It publishes actual charges
only after successful construction; serialized boot context prevents intervening
allocation or rebudgeting. All partial image, stack, runtime-table and device-table
allocations roll back on failure. An admitted task pays for its executable pages,
stack and all private page tables, including tables prepared for runtime buffers.
Framebuffer device pages are not RAM backing, but their mapping tables are charged.

[Runtime allocation](../../memory/allocation.rs) reserves the
whole object's frame charge before allocating/zeroing/mapping. Failure returns
both partial backing and unused charges. A reader pays its own mapping tables at
admission and its separate accepted-reader quota, but does not pay again for the
producer's physical backing. The region's persistent sponsor keeps that charge
after producer exit, until the final accepted reader releases it or dies.

Task retirement refunds only the address space. Retained backing remains charged
and prevents withdrawal below its charge. Dynamic launch release checks all
resource families before returning any funding. Account reuse never resets live
charges. Launch loans and ancestor return carry the same reserved account; unused
funding is returned explicitly rather than created anew at every tree level.

The frame bank has no fallible metadata growth after preparation. IPC arenas,
kernel heap/boot allocations and permanently reserved hardware storage remain
separate fixed kernel costs. Enough frame funding does not promise that other
bounds (image size, objects, mappings, handles or process slots) permit an operation.
There is no demand paging, swap, active-child rebudgeting or sponsorship transfer.

## Evidence and matrix integration

Host tests cover physical bank exhaustion/rollback, nested conservation, sibling
capacity and retained charges. `--smoke` tests every load allocation boundary with
both injected failure and actual funding limits, runtime allocation rollback,
producer restart with an accepted reader, refused funding withdrawal, final-reader
refund and complete physical/heap reclamation.

`--admission-test` exhausts a nested reservation while an outside sibling allocates
and communicates, retries an underfunded load, replenishes it and cancels/reuses
the subtree four times without changing baseline funding.

The matrix desktop gives its root the physical funding for its process pool.
Owner-only `RUN_MEMORY` stages a frame allowance for the next `RUN` attempt on an
empty slot; that attempt consumes the setting even if admission fails. Default
`RUN` uses 256 frames per requested process account. Children and physical input
connections cannot issue this policy operation. The desktop probes failed nested
loads while the outside app removes/restores a surface and real pixels change,
then replaces the inner matrix with enough funding and continues ordinary input.

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 120
python tools/boot-harness-rs/run.py --admission-test --release --timeout 120
python tools/boot-harness-rs/run.py --matrix-desktop-test --release --timeout 120
```
