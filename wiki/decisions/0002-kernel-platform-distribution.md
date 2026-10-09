# ADR 0002: Kernel, platform and one distribution

**Accepted 2026-10-09.** Supersedes ADR 0001's flat grouping and placement of
distribution composition outside target source. Other provenance, contract
ownership and testing principles remain in effect.

Cathedral ships a kernel, a common OS platform outside the kernel, and one
built-in distribution in one monorepo. Forks can replace `source/distribution/`;
Cathedral does not need a multi-distribution framework.

Both source trees group boot/core/CPU work under `kernel/` and services/drivers/
libraries under `platform/`. Contracts and kernel-safe foundation stay shared.
The singular `distribution/` owns shell, settings, applications, defaults and
composition. Its first resident is the boot profile read by the host harness.

Dependencies flow from distribution to platform, never back. Userspace targets
contracts, not kernel internals. Standard capture/input services belong to the
platform; appearance and workflows belong to the distribution. Trusted owners
continue to enforce grant and recovery semantics.

This move does not claim userspace isolation. Rust retains one UEFI/kernel image,
the upstream UEFI crate and thin entry orchestration. Boot still links the UART,
with port operations supplied by the kernel rather than imported by the driver.

The [layout document](../architecture/repository_layout.md) owns the tree and
dependency rules; host checks enforce the implemented boundaries.
