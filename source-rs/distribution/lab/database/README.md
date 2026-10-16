# Funded heap and guest database proof

[main.rs](main.rs) supervises three executions of the independently linked
[worker](worker/main.rs), selected by [database.json](../../../../tools/boot-harness-rs/profiles/database.json).
It withdraws and restores the worker's 64-page reservation after a leaked allocation
exit, a contained page fault, and normal completion. A retained page charge makes
that withdrawal fail. The worker installs its heap, checks exhaustion, then enters
[transaction.rs](worker/transaction.rs) to open and commit the database.

From the repository root:

```powershell
python tools/boot-harness-rs/database_test.py
python tools/boot-harness-rs/database_test.py --release
```

The runner formats a new guarded 4 MiB raw region on the host. The worker holds the
existing exclusive ATA controller grant and runs redb 4.4.0 without std, using a
32 KiB cache and a 256 KiB heap. Three lab records represent one atomic
record/head/event publication; this is not yet the DBFS schema or storage protocol.
After acknowledging revision 1 it yields indefinitely with the database still open.
The runner kills QEMU, verifies a copy with host redb, and boots the original image.
The second guest recovers revision 1, commits revision 2, closes the engine and
checks zero live heap allocations. The host independently verifies all three records
and unchanged bytes outside the volume after each boot.

Artifacts are unique under `build/boot-harness-rs/database/`: the raw image, host
verification copies, serial/QEMU logs, source hashes and check context. The general
profile runner permits only build-only use; it cannot accidentally use a session disk.
QEMU kill preserves host caches: this is an unclean guest restart test, not a physical
power-loss simulation. The older 939-cut host engine/envelope sweep is separate.

## Measured limits

The [host allocator meter](../../../../tools/storage-lab/engines/portability/README.md)
observed about 100 KiB peak requested live bytes with a 32 KiB cache. Allocator
metadata, fragmentation and recovery need additional headroom. The guest receives
64 pages explicitly; it cannot grow silently. A 512 KiB fallible allocation fails,
and one extra page is refused by the account's quota.

The worker uses 208 loaded pages in release and 233 in the initial size-optimized
dev build. Both redb and the worker use size optimization and disable debug_assert!
in dev; explicit assert! checks remain enabled. Worker DWARF is stripped to fit
the existing ELF-file cap. Admission permits 240 image pages, still within the
1 MiB virtual window; 288 frames cover image, stack and translation overhead.
Unclean recovery hit the former 16 KiB stack guard. User stacks now have 32 KiB
of backing with the same guard boundary. These are bounded lab limits, not measured
production requirements or guarantees for arbitrary transactions.

The shared kernel bookkeeping now stores live frame/backing vectors with fallible
reservation before mutation; write permission is recorded per backing instead of
in a u128 mask. Runtime object frame vectors release capacity on retirement. The
normal x86/ARM64 proofs check rollback, isolation and complete reclamation.

The engine is still a candidate. No guest formatter, production DBFS schema,
multi-client request scheduling, larger workload bound, or new catalog disk format
is introduced. Next is to drive bounded DBFS-shaped publications through this
runtime and measure allocation/recovery pressure as the database grows.

## Validation recorded on 2026-10-16

Debug database proof: `20261016T234240824584Z`; release:
`20261016T233708775024Z`, both under `build/boot-harness-rs/database/`.
Workspace host tests, three engine-backend tests, two allocator tests, x86 target
lint, ARM64 engine/allocator lint, full x86 debug smoke (240-second timeout),
ARM64 debug portability and source-layout checks passed. The full smoke also
verifies copy rejection at the updated user-stack guard boundary.
