# Kernel memory protection

After activating owned page tables, the kernel image has executable/read-only
code, read-only/non-executable data, and writable/non-executable mutable data.
Headers and padding are read-only/non-executable. Heap, boot stacks, emergency
stacks, task stacks, allocator RAM and device windows are non-executable.
Lower and upper stack guards remain unmapped.

## Follow the handoff

1. [Firmware entry](../../boot/uefi/firmware/firmware.rs) asks the UEFI crate for
   the loaded image. [Its image adapter](../../boot/uefi/firmware/image.rs) reads
   the relocated PE32+ section headers; UEFI still owns loading and relocation.
   The adapter rejects unsupported geometry and more than sixteen sections.
2. [KernelImage](image.rs) validates page-separated, ordered sections and rejects
   overlapping or writable/executable sections. It carries physical permission
   intent without importing PE or either ISA.
3. [Boot memory composition](../../boot/uefi/memory.rs) supplies that description
   and fresh frames to the selected backend. Preparation builds an inactive tree;
   [handoff](../../boot/uefi/handoff.rs) activates it before entering the new stack.

On [x86-64](../isa/x86_64/memory/memory.rs), `inherited.rs` retains firmware mapping
topology and leaf cache attributes, owns every table, and imposes permissions by
physical backing. Every alias of an image page gets its section permissions.
Large leaves overlapping the image split down to 4-KiB pages, preserving PAT
encoding. Other leaves become supervisor-only/NX. Activation enables CR0.WP and
EFER.NXE and flushes inherited global translations, not only non-global entries.
User roots copy these protected image permissions.

On [ARM64](../isa/aarch64/memory/memory.rs), `identity.rs` maps board windows NX
and splits image-containing 2-MiB blocks into 4-KiB pages. Only image code permits
EL1 execution; all kernel mappings deny EL0 execution. SCTLR.WXN additionally
prevents execution through writable mappings. User roots borrow this same
supervisor branch. Heap/stack virtual aliases and their RAM identity aliases
both deny execution.

## Evidence and boundaries

[Fatal probes](../../boot/uefi/smoke/protection.rs) execute one forbidden access
per QEMU boot. They require the exact address and architectural permission-fault
syndrome, including instruction-fetch faults for NX checks. Both runners accept
`--fault` with `text-write`, `rodata-write`, `heap-execute`, `stack-execute`,
`heap-alias-execute`, `stack-alias-execute`, and `guard`:

```text
python tools/boot-harness-rs/run.py --smoke --kernel-only --release --fault heap-alias-execute
python tools/boot-harness-rs/arm64/run.py --kernel-only --release --fault heap-alias-execute
```

Host tests reject invalid image permissions and exercise two synthetic x86
aliases of the same image inside 1-GiB mappings, including PAT-preserving splits.
Ordinary debug/release boot suites verify that scheduling, exceptions, user
admission and reclamation still work with the protected image.
The standalone firmware parser test also rejects truncated headers, address
overflow, section overlap and unsupported PE layouts. From `source-rs/` (after
creating the repository's `build/` directory):

```text
rustc --edition 2024 --test kernel/boot/uefi/firmware/image_test.rs -D warnings -o ../build/uefi-image-tests.exe
../build/uefi-image-tests.exe
```

That host test root imports the actual shared permission types; it does not
build or run the UEFI entry point. The normal architecture tests run with
`cargo test --locked`.


Protection starts when owned mappings activate, not at firmware entry. The old
firmware stack and storage remain reserved; no runtime firmware calls are made.
ARM briefly disables translation while replacing its firmware regime, executing
through the identity image. That bootstrap interval precedes enforcement.

This hardens the kernel image and denies execution from kernel data mappings.
Privileged code can still edit writable page tables, access allocator RAM through
identity aliases, and load or copy user images through writable backing. It is
not isolation between trusted kernel threads, a general immutable-memory object
contract, or W^X across every user-code/backing alias. There is no dynamic kernel
module loader, reclamation of firmware storage, SMP shootdown protocol, or ARM
kernel stack-overflow recovery in this milestone.
