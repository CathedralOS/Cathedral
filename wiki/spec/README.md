# Cathedral specification

This directory defines Cathedral's current accepted contracts, organized by
subject. The specification uses definitions, rules, state transitions, failure
outcomes, and conformance evidence where useful. It is not claimed to be
complete or fully formalized.

An unwritten subject is unspecified even when a design chapter describes a
strong intended direction. A partially covered subject is normative only for
the rules it actually states. Implementation details remain beside code.

## Status model

Every specification page reports three independent facts:

- **Contract status** is `Current` for accepted rules. Candidate rules belong in
  [proposals](../proposals/README.md), not under `spec/`.
- **Specification coverage** is `Partial` or `Covered`. `Partial` means the
  stated rules are current but do not yet close the whole named subject.
- **Implementation coverage** is `None`, `Partial`, or `Complete`, with links to
  the evidence used for that assessment.

`Covered` is a strong claim and must identify the complete boundary of the
subject. A current specification is allowed to precede implementation.

## Current specification subjects

| Subject | Specification | Specification coverage | Implementation coverage |
| --- | --- | --- | --- |
| UEFI boot-services ownership transition | [UEFI boot-services ownership transition](boot/uefi_boot_services.md) | Partial | Partial |
| Initial root extent | [Initial root extent](resources/root_extent.md) | Partial | Partial |

## Owned but unwritten subjects

This table establishes documentation ownership without pretending that an
empty page is a contract. These are the frozen-semantics subjects identified by
the [governance design](../design/part_7_governance/04_governance_and_extension_boundaries.md),
plus the active hardware frontier.

| Subject | Intended specification owner | Current design owner | Implementation state |
| --- | --- | --- | --- |
| Capability meaning, delegation, attenuation, redemption, and revocation | `spec/authority/capabilities.md` | [Capability model](../design/part_1_authority/00_capability_model.md) and [lifecycle](../design/part_1_authority/01_capability_lifecycle.md) | Not started |
| Components, Matrices, manifests, lifecycle, and host-chain semantics | `spec/execution/components_and_matrices.md` | [Component model](../design/part_2_components/00_component_model.md) | Not started |
| IPC endpoints, shared regions, protocols, and transfer | `spec/communication/ipc.md` | [IPC and service invocation](../design/part_3_communication/00_ipc_and_service_invocation.md) | Not started |
| Realm and filesystem object semantics | `spec/storage/realms_and_objects.md` | [Filesystem as database](../design/part_4_storage/00_filesystem_as_database.md) | Not started |
| Package closure and component manifest | `spec/packages/package_and_manifest.md` | [Package system](../design/part_5_lifecycle/00_package_system.md) | Not started |
| Compositor and seat contract | `spec/human_surface/compositor_and_seat.md` | [Windowing and compositor](../design/part_6_human_surface/00_windowing_and_compositor.md) | Not started |
| Driver contract | `spec/devices/drivers.md` | [Driver model](../design/part_5_lifecycle/02_driver_model.md) | Hardware fact packages only |
| Checker and installation admission | `spec/admission/checker.md` | [Kernel architecture](../design/part_5_lifecycle/04_kernel_architecture.md) | Partial Omega admission path |
| x86-64 page-table, exception, interrupt, and timer bootstrap policy | `spec/architecture/x86_64_bootstrap.md` | [Hardware foundation profile](../design/part_0_foundations/03_hardware_foundation_profile.md) | Pure policy and validation helpers; installation remains absent |

## Adding or changing a contract

1. Audit the existing specification owners and machine-readable contracts.
2. If the rule is accepted, write the specification change before or alongside
   its implementation.
3. If a concrete requirement exposes an unresolved semantic, compatibility, or
   trust choice, raise it in [`OWNER_QUESTIONS.md`](../../OWNER_QUESTIONS.md).
4. If behavior is merely a candidate, place it in
   [proposals](../proposals/README.md).
5. Link conformance evidence without making generated build output normative.

Specification pages do not contain open-ended design-question sections. They
may identify an explicit exclusion or an unestablished claim, but alternatives
and research remain in design, proposals, or drafts.
