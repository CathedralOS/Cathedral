# CHARTER — `source/contracts/`

**Scope.** The frozen ABI everything targets: capability vocabulary, the
kernel/syscall surface, IPC wire schemas, the boot handoff, the component +
manifest format, the checker's admission contract. Foreign ABIs Cathedral must
match at a boundary (the UEFI hand-off, later PCI/ACPI shapes) also live here —
transcribed from their primary specs as plain `data` given byte layout by a
layout policy at the use site.

**Depends on.** Nothing. `contracts/` is a root of the reach graph — everything
depends on it; it depends on nothing.

**Non-goals.** No executing behavior beyond the definitions themselves; no
implementations (those live in `core/`, `services/`, `drivers/`). Contracts are
*what everyone agrees on*, not *what anyone does*.

**Evolution.** Changes only by the versioned-interface discipline (additive +
migrated, never redefined in place). A change here redefines the platform.

**Status (2026-07-02).** In-progress: `uefi/` (the milestone-1 boot hand-off ABI).
