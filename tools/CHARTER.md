# CHARTER — `tools/`

**Scope.** Host-side tooling that runs on the developer's machine and **never
ships** to the target: SDK / build glue, the reference IDE, the debugger, the
hostile simulator, CI gate runners, image assembly, and the QEMU/OVMF boot
harness. If it runs on the developer's machine rather than on Cathedral, it is
here.

**Depends on.** Nothing in the OS build graph. `tools/` is outside `source/`; it
may *read* `source/contracts/` for schema-awareness, but it ships nothing and
nothing in `source/` depends on it.

**Non-goals.** No code that runs on the Cathedral target (that is `source/`). No
capability-holding OS logic.

**Status (2026-07-02).** In-progress: `boot-harness/` — build the milestone-1
UEFI application and boot it under QEMU/OVMF. The QEMU invocation is real; the
build step is stubbed until the Omega toolchain can emit a UEFI target
(first-boot ladder, milestone 1).
