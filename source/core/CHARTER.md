# CHARTER — `source/core/`

**Scope.** The proved kernel — the trusted computing base, and nothing more: the
capability arena, scheduler, memory/address-space manager, IPC, timers, trusted
time, spawn/loader, hot-swap engine, transaction coordinator, attestation, and
the proof checker. Small enough to audit in full. Proofs live **here**, beside
the code they cover — never in a sibling that can lag.

**Depends on.** `foundation/` and `contracts/` only. **Nothing depends on
`core/` at build time** — userspace targets the ABI in `contracts/`, which
`core/` implements, so `core/` is a leaf in the build graph. `core/` stays
firmware-neutral: the UEFI-specific memory-map walk is `boot/`'s job; `core/`
mints `Region`s from firmware-neutral spans.

**Non-goals.** No device drivers, no userspace service logic, no firmware ABIs.
If it isn't small enough to audit in full, it doesn't belong here.

**Status (2026-07-04).** In-progress: `region.omg` — milestone 2, the first
capability (`Region`, exclusive authority over physical RAM) and its mint, the
origin of the authority graph. The generational `{slot, generation}` arena that
makes capabilities unforgeable + revocable is the `capability_lifecycle` arc,
later.
