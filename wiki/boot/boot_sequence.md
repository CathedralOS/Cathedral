# Boot Sequence

> A step-by-step walkthrough of how a Cathedral machine comes up, from power-on to the first usermode process. This is an **explainer**, not a design chapter: it linearizes a sequence that crosses several design chapters and shows the order things happen in.
>
> Status: **intended mechanism.** There is no implementation yet. This describes how the design is meant to boot, and it will change as the design firms. Each step links to the design chapter that owns its contract.

## The one hard problem

Almost everything on disk is content-addressed: you find an object by its hash, and a directory is just an object listing its children by hash ([filesystem](../design/part_4_storage/00_filesystem_as_database.md)). That creates a chicken-and-egg at boot: to read anything you need the current root hash, but the root hash is itself on the disk you can't read yet. The **superblock** is the one fixed-location thing that breaks the cycle. Everything below is the path from "machine powered on" to "inside the content-addressed world."

## Step by step

**1. Firmware / UEFI (you don't control this).**
Power on. UEFI finds the ESP (a plain FAT32 partition), loads your kernel image from it, and jumps to it. Two things to notice: the kernel image has to live in a filesystem the firmware can read (FAT), because firmware cannot read your content-addressed store, so the ESP is a *second* fixed anchor separate from the superblock. UEFI also tells the kernel which device it was loaded from (the boot disk) and hands over hardware description (ACPI tables).

**2. Kernel entry, very early.**
The kernel runs privileged (ring 0 on x86, EL1 on ARM). On x86_64 the firmware already left paging on with a dumb identity map, so it is not "no virtual memory," it is "replace the firmware's identity map with your own page tables and set up a kernel heap." At some point the kernel calls `ExitBootServices()`, which is the line between borrowing UEFI's disk driver and taking over the disk with your own. See [kernel architecture](../design/part_5_lifecycle/04_kernel_architecture.md) and [memory & persistence](../design/part_2_components/02_memory_and_persistence.md).

**3. Read the superblock.**
On the boot disk, at a known offset, sits the superblock: a tiny `wire data` header carrying a `format_version` (so an old kernel refuses a too-new volume), `root_ref` (the hash of the current root object), and `log_head` (where the append-only log ends). It is written redundantly as a small ring of generation-numbered slots, written atomically, so a crash mid-update never loses the volume: on boot you scan the slots and pick the highest valid generation. This is the only fixed-location structure in your store, and it is the same idea as git's `refs/heads/main` or ZFS's uberblock. See the superblock note in [filesystem](../design/part_4_storage/00_filesystem_as_database.md).

**4. Turn hashes into locations (the step that is easy to miss).**
`root_ref` is a *hash*, not a *location*. To read that object you must first know where on the disk that hash's bytes live, which is the `hash -> location` index. So the real bootstrap is the **log**: it sits at a self-describing position (`log_head`), and its records tell you where objects were placed. You read or replay the log to learn locations and rebuild the index, and only then can you resolve `root_ref`. The index is a rebuilt accelerator, not the source of truth; a persisted index checkpoint lets you skip replaying the whole log and replay only the tail. See "the transaction log as the source of truth" in [filesystem](../design/part_4_storage/00_filesystem_as_database.md).

**5. Enter the content-addressed world.**
With the index in hand, resolve `root_ref` to the root object. That is the apex of the object graph; the realm registry and the system realm hang off it. From here everything works the way the design describes: walk the Merkle DAG by fetching child hashes, the object store plus the index resolve each hash to a physical extent, and objects can live on any drive or be replicated across drives. The hierarchy is not tied to "this drive is this filesystem."

**6. Reach the latest committed state.**
The superblock may lag slightly behind the true log head (superblock updates are batched), and a crash may have left a partial transaction. Replay the log forward from the superblock's recorded position to the last fully committed transaction. After this the live root is consistent.

**7. Normal Cathedral life starts.**
The kernel object cache warms. Full virtual memory and the single-address-space Omega kernel are running. The capability and realm machinery layers on top: create the initial realms, hand the first components their root capabilities ([capability model](../design/part_1_authority/00_capability_model.md), [realms](../design/part_4_storage/00_filesystem_as_database.md)). Eventually you start real usermode processes inside realms.

## Why the superblock feels out of place

Because it is the one traditional, fixed-location thing in an otherwise content-addressed system. All the Merkle-DAG machinery has a chicken-and-egg at boot ("I need the root hash to read anything, but the root hash is on the disk I can't read yet"), and the superblock is the external pointer that lets you in. Once you are inside, the root hash plus the log plus the objects are self-describing and you never need another fixed-location structure. git needs `refs/heads/main`; ZFS needs an uberblock; Cathedral made that pointer as small as possible.

## See also (design contracts)

- [Boot, Trust Chain & Recovery](../design/part_5_lifecycle/03_boot_and_trust_chain.md) — the measured-boot and recovery contract this sequence runs inside.
- [Kernel Architecture](../design/part_5_lifecycle/04_kernel_architecture.md) — what is privileged, and OS-vs-app isolation.
- [Filesystem as Database](../design/part_4_storage/00_filesystem_as_database.md) — the object store, the superblock, the log, content addressing.
- [Memory & Persistence](../design/part_2_components/02_memory_and_persistence.md) — the single-level store and early memory setup.
