# Phase 4: Mounting the Object Store

> Crossing from raw disk reads into the content-addressed world: the superblock, the log that turns hashes into locations, and the root of the authority tree. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

Nearly everything on disk is content-addressed: an object is found by the hash of its bytes, and a directory is itself just an object listing its children by hash ([filesystem](../design/part_4_storage/00_filesystem_as_database.md)). That creates a bootstrap problem. To read anything you need the current root hash, but the root hash is stored on the disk you cannot read yet. The superblock breaks the cycle.

## The superblock

At a fixed offset on the boot disk sits the superblock: a small record (in the versioned `wire data` serialization the rest of the store uses) holding a format version, `root_ref` (the hash of the current root object), and `log_head` (the end of the append-only log). It is written as a ring of slots, each tagged with a generation number and updated atomically, so a crash mid-write cannot destroy it: on mount you read the slots and take the highest valid generation. It is the only structure in the store kept at a fixed location, and it plays the same role as a Git ref or a ZFS uberblock.

## A hash is not a location

`root_ref` is a hash, not a disk address. To read that object you first have to know where its bytes physically sit, which is the job of the `hash -> location` index. So the real bootstrap is the **log**. It lives at a position the superblock records (`log_head`), and its entries record where each object was written, so reading or replaying the log yields the locations and rebuilds the index. Only then can `root_ref` be resolved. The index is a rebuilt cache, not the source of truth; a saved checkpoint lets you replay only the log tail instead of the whole thing.

## Reaching the live root

Resolving `root_ref` gives the root object, the top of the object graph. The superblock can lag slightly behind the true end of the log, because its updates are batched, and a crash may have left a half-written transaction, so the kernel replays the log forward from the superblock's recorded position to the last fully committed transaction. After that the live root is consistent.

## The authority tree comes online

The root object holds the **realm registry**, the single node above every realm ([filesystem](../design/part_4_storage/00_filesystem_as_database.md)). The kernel takes the registry's authority, which is the root of all authority on the system, and the system realm becomes reachable. The capability machinery from [phase 3](03_kernel_subsystems.md) now has its anchor, and component code, which lives in the system realm, can finally be loaded.

## Next

[Phase 5: Components and services](05_components_and_services.md).
