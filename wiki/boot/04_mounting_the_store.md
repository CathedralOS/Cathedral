# Phase 4: Mounting the Object Store

> Crossing from raw disk reads into the content-addressed world: the superblock, the log that turns hashes into locations, and the root from which the realm registry hangs. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

Almost everything on disk is content-addressed: you find an object by its hash, and a directory is an object listing its children by hash ([filesystem](../design/part_4_storage/00_filesystem_as_database.md)). So there is a chicken-and-egg: to read anything you need the current root hash, which is on the disk you cannot read yet. The superblock breaks it.

## The superblock

At a known offset on the boot disk sits the superblock: a tiny `wire data` header with a `format_version` (so an old kernel refuses a too-new volume), `root_ref` (the hash of the current root object), and `log_head` (where the append-only log ends). It is written as a small ring of generation-numbered slots, updated atomically, so a crash mid-update never loses the volume: on mount you scan the slots and pick the highest valid generation. It is the only fixed-location structure in the store, and it is the same idea as git's `refs/heads/main` or ZFS's uberblock.

## A hash is not a location

`root_ref` is a hash, but to read that object you must know where its bytes physically live, which is the `hash -> location` index. So the real bootstrap is the **log**: it sits at a self-describing position (`log_head`) and its records say where objects were placed. You read or replay the log to learn locations and rebuild the index, and only then can you resolve `root_ref`. The index is a rebuilt accelerator, not the source of truth; a persisted checkpoint lets you replay only the tail rather than the whole log.

## Reaching the live root

Resolve `root_ref` to the root object, the apex of the object graph. The superblock may lag slightly behind the true log head (its updates are batched) and a crash may have left a partial transaction, so replay the log forward from the superblock's recorded position to the last fully committed transaction. After this the live root is consistent.

## The realm registry comes online

The root object is where the **realm registry** lives, the single privileged node above all realms ([realms](../design/part_4_storage/00_filesystem_as_database.md)). The kernel takes the realm-authority for the registry, which is the root of all authority in the system, and the system realm becomes reachable. The capability and realm machinery from [phase 3](03_kernel_subsystems.md) now has its anchor: there is a top of the authority tree, and component code (which lives in the system realm's package store) can finally be loaded.

## Next

[Phase 5: Components and services](05_components_and_services.md) — starting the OS's own processes from the system realm.
