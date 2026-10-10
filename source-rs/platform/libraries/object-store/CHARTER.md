# Atomic private-object lab

This platform library owns recovery and atomic replacement over the shared
`block::Device` read/write/flush contract. It imports no kernel implementation.
It stores one connection-relative object (1), at most 32 bytes, with a monotonic
generation. A replace supplies the expected generation; stale attempts fail.
There are no paths, global object lookup, delegation or ambient object rights.

Four 512-byte sectors hold alternating `(body, commit)` pairs. A body contains
`CTOBJ001`, generation, length, object number, payload, zero padding and CRC32.
A commit contains `CTCOM001`, generation, body CRC, zero padding and its own
CRC32. All integers are little-endian; CRCs occupy bytes 508–511. Replacement
writes the inactive body, flushes, writes its matching commit, flushes again,
then acknowledges. Recovery selects the highest complete valid pair. Equal
valid generations are rejected. CRC is accidental-corruption detection, not a
content address, cryptographic integrity guarantee or proof against every tear.

A new all-zero four-sector region receives a flushed generation-zero empty
baseline before serving clients. Other unrecognized media is never reformatted.
Interrupted initial formatting can leave no valid root and fail closed; the lab
does not yet repair it. Once a baseline exists, interruption of replacement
retains a complete old or new version. An ambiguous device error poisons the
instance until reopened. A lost reply does not undo a committed replacement;
clients read/reconcile before retrying with an expected generation.

Durability assumes the backend honors flush and preserves previously flushed
sectors outside the write target. It does not cover malicious media, unrelated
sector corruption or a lying cache. The host tests drop volatile state and tear
each write/failed flush at every byte prefix, including the first replacement.
The QEMU harness kills the emulator at four write boundaries and checks reboot
state; this is guest power loss, not physical host power-loss certification.

This is an experimental format, not Cathedral's filesystem DB. No catalog,
typed schema, content-addressed blobs, queries, snapshots, replication, GC,
multi-object transactions or kernel/storage distributed commit exists yet.
Those remain platform work; kernel authority and lifecycle mechanisms remain
independently usable without this library or its service.
