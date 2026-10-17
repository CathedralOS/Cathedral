# Omega boot frontier

This directory contains Cathedral's boot source and a bounded physical-entry
ladder. `build.omg` selects `CathedralBootstrap` for firmware arrival and binds
the target's semantic slot to `Kernel::enter(image, initial_storage)`.
`physical_entry.omg` is written to validate the incoming table header, print
`Hello from Omega`, and return `EFI_UNSUPPORTED`; it does not claim a successful
handoff or fabricate the two storage roots. The preserved map/exit/report
implementation in `own_machine.omg` is not yet reached by that adapter.

The source selection passes ordinary package review and reaches native emission,
but is **not native-build- or boot-verified**. The current refusal is native
settlement of the text-output wrapper's direct foreign leaf: the selected
boundary provider is retained, but its explicitly called external dependency
has no selected realization row. The leaf needs its own exact binding custody;
aliasing it to the wrapper's boundary slot would recurse into the wrapper.

The following known refusal is the physical entry stack check: Omega assigns
every foreign call a fixed 256 KiB same-stack
contribution, exceeding the UEFI target's 128 KiB arrival guarantee before the
adapter's own frames and reserve are added. This needs admitted per-provider
demand, not a smaller unexplained constant or a larger invented guarantee.
Historical boot comments and old images do not establish that this tree runs.

The intended acceptance is one freshly compiled `uefi_x86_64` image under
QEMU/OVMF: `Hello from Omega`, successful firmware exit, `Owned N MiB` over
COM1, then idle while retaining the granted root. A failed build or an old
`BOOTX64.EFI` cannot satisfy acceptance.

## Reproduce

Use an optimized current Omega checkout or pass its exact binary:

```text
python tools/boot-harness/run.py --compiler PATH_TO_OMEGA --prepare --build-only
python tools/boot-harness/run.py --compiler PATH_TO_OMEGA --smoke
```

`--prepare` invokes ordinary package review; it does not accept trust grants.
Follow the emitted review instructions before attempting the build again.
The accepted `omega.lock` is produced by successful checked review, not
hand-authored acceptance records. A different compiler identity requires ordinary
review again. The current local-source lock also identifies its checkout path;
regenerate it in the receiving checkout rather than committing this worktree's
acceptance as if it were portable.

## Work still required

- Close directly called foreign declarations into the selected realization
  dependency set, preserving exact leaf identity, target, calling plan, trust,
  and table-field provenance. The inline implementation calls an ordinary
  helper; removed conformance assignment syntax is not a workaround.
- Replace the compiler's fixed foreign-call stack contribution with the exact
  selected provider's admitted demand, then rerun the fresh-image harness.
  Ordinary package admission now publishes the five-package lock without
  blanket-accepting unrelated host capabilities.
- Connect the checked physical firmware adapter to the semantic entry and
  Cathedral continuation: obtain the image through LoadedImage protocol
  discovery, establish bootstrap stack/storage custody, validate nonoverlap,
  and retain the successful final-map/ExitBootServices provenance for both
  semantic arguments. `Main::run` remains ordinary checked logic, not an
  implicitly installed raw ABI. PE subsystem selection, a same-shaped pointer,
  or an empty root provider is not proof of firmware validity or authority.

The std pin includes the borrowed UTF-8 calling-policy rejection text. Older
revisions declared a fixed 256-byte field but initialized it with shorter
literals, which rejects before this program's entry can be selected.

The local calling policy deliberately remains a normal named implementation;
it must not collide with std's same-spelled policy. Firmware layouts and OS
map-selection/retry policy stay in authored Omega, not compiler intrinsics.

Firmware calls use static boundary requirements with explicitly selected named
providers. `Main` stores only invocation data; the console/table borrows, image
handle and map key remain ordinary arguments. There are no `Binding` fields or
implicit provider arguments. Selecting the implementations does not establish
the validity or lifetime of those firmware values.

The Rust bring-up lab and the returning physical-entry ladder do not discharge
the full acceptance. Keep the full smoke harness failing visibly until this
exact source produces a fresh image, exits firmware, and reports ownership.
