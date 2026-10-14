# Omega boot frontier

This is the real Cathedral boot source, not a reduced hello-world surrogate.
The five-package candidate graph now passes checked compilation with the
compiler's static Binding and numeric state-contract repairs. It is **not
native-build- or boot-verified**.
Historical July boot comments do not establish that the current tree runs.

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
The obsolete text-format `omega.lock` was removed; a replacement must come
from a successful checked review, not hand-authored acceptance records.

## Work still required

- Complete ordinary package admission. `omega update --project
  source/kernel/boot/uefi --target uefi_x86_64` reaches the candidate review;
  it does not yet install an accepted lock. The pinned std package also reports
  unrelated host capabilities: do not blanket-accept its pending rows merely
  to make the boot command proceed.
- Connect the checked physical firmware adapter to the semantic entry and
  Cathedral continuation. Omega's `UEFI-PHYSICAL-SEMANTIC-ENTRY` and
  `UEFI-OS-HANDOFF` outcomes own the general compiler/target-source mechanics.
  `Main::run` is ordinary checked logic, not an implicitly installed raw ABI.
  PE subsystem selection, a same-shaped pointer, or an empty root provider is
  not proof of firmware validity or storage authority.

The local calling policy deliberately remains a normal named implementation;
it must not collide with std's same-spelled policy. Firmware layouts and OS
map-selection/retry policy stay in authored Omega, not compiler intrinsics.

The Rust bring-up lab is a separate product and does not discharge this
acceptance. Keep the harness failing visibly until this exact source produces
and boots a fresh image.
