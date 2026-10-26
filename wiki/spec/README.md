# Cathedral specification

This directory defines Cathedral's current accepted contracts, organized by
subject. The specification uses definitions, rules, state transitions, failure
outcomes, and conformance evidence where useful. It does not claim to be
complete or fully formalized.

An unwritten subject is unspecified even when a design chapter describes a
strong intended direction. A partially covered subject is normative only for
the rules it states. Implementation details remain beside code.

## Status model

Every specification page reports three independent facts:

- **Contract status** is `Current` for accepted rules. Candidate rules belong in
  [proposals](../proposals/README.md), not under `spec/`.
- **Specification coverage** is `Partial` or `Covered`. `Partial` means the
  stated rules are current but do not yet close the whole named subject.
- **Implementation coverage** is `None`, `Partial`, or `Complete`, with links to
  the evidence used for that assessment.

`Covered` is a strong claim and must identify the complete boundary of the
subject. A current specification may precede implementation.

## Current specification subjects

| Subject | Specification | Specification coverage | Implementation coverage |
| --- | --- | --- | --- |
| UEFI physical entry and Cathedral semantic handoff | [UEFI physical entry and Cathedral semantic handoff](boot/uefi_entry_handoff.md) | Partial | Partial |
| UEFI boot-services ownership transition | [UEFI boot-services ownership transition](boot/uefi_boot_services.md) | Partial | Partial |
| Initial root extent | [Initial root extent](resources/root_extent.md) | Partial | Partial |
| Execution budgets and service work | [Execution budgets](resources/execution_budgets.md) | Partial | Partial |
| Hierarchical backing budgets, direct delegation and allocator interception | [Memory budgets](resources/memory_budgets.md) | Partial | Partial |
| Realms and objects: copies, paths and hierarchy metadata | [Realms and objects](storage/realms_and_objects.md) | Partial | Partial |
| Object-rooted live watches and transaction-coupled durable capture | [Change watches](storage/change_watches.md) | Partial | None |
| Explicit revision prerequisites and atomic multi-object mutations | [Atomic storage batches](storage/atomic_batches.md) | Partial | None |
| Reference deletion, retention policy and scoped cryptographic erasure | [Deletion and erasure](storage/deletion_and_erasure.md) | Partial | None |
| Components and matrices: policy ownership and supplied authority | [Components and matrices](execution/components_and_matrices.md) | Partial | Partial |
| Independent installation, notification and component-owned activation | [Installation and activation](execution/updates.md) | Partial | None |
| Hardware separation and exclusion of PCC-based single address space | [Program isolation](execution/isolation.md) | Partial | Partial |
| Scoped coordination and checked shared-memory IPC | [Coordination and IPC](communication/ipc.md) | Partial | Partial |
| Capability-controlled signals, waits and combined signal-and-wait | [Notifications and parking](execution/notifications.md) | Partial | None |
| Replaceable desktop programs and protected outer recovery chrome | [Compositor and seat](human_surface/compositor_and_seat.md) | Partial | Partial |
| Scoped input delivery, representations, interaction targets and mappings | [Input delivery and interaction](human_surface/input.md) | Partial | Partial |
| Logical input channels, stable identity and scoped discovery | [Input channels and discovery](human_surface/input_channels.md) | Partial | None |
| Exclusive focus-group control, recursive local selection and input transitions | [Focus groups and routes](human_surface/focus_routes.md) | Partial | Partial |
| Owner-managed placement, rendering demand, scoped fullscreen and backend boundaries | [Rendering and presentation](human_surface/rendering.md) | Partial | Partial |
| Authorized image versions, local/remote backing and safe reclamation | [Rendering resources](human_surface/rendering_resources.md) | Partial | Partial |
| Shared-buffer playback, policy hierarchy and final mixing | [Audio playback](human_surface/audio.md) | Partial | None |

## Owned but unwritten subjects

This table assigns documentation ownership. An empty page is not a contract.
These are the frozen-semantics subjects identified by the
[governance design](../design/part_7_governance/04_governance_and_extension_boundaries.md),
plus the active hardware frontier.

| Subject | Intended specification owner | Current design owner | Implementation state |
| --- | --- | --- | --- |
| Capability meaning, delegation, attenuation, redemption, and revocation | `spec/authority/capabilities.md` | [Capability model](../design/part_1_authority/00_capability_model.md) and [lifecycle](../design/part_1_authority/01_capability_lifecycle.md) | Not started |
| Package closure and component manifest | `spec/packages/package_and_manifest.md` | [Package system](../design/part_5_lifecycle/00_package_system.md) | Not started |
| Driver contract | `spec/devices/drivers.md` | [Driver model](../design/part_5_lifecycle/02_driver_model.md) | Hardware fact packages only |
| Checker and installation admission | `spec/admission/checker.md` | [Kernel architecture](../design/part_5_lifecycle/04_kernel_architecture.md) | Partial Omega admission path |
| x86-64 page-table, exception, interrupt, and timer bootstrap policy | `spec/architecture/x86_64_bootstrap.md` | [Hardware foundation profile](../design/part_0_foundations/03_hardware_foundation_profile.md) | Pure policy and validation helpers; installation remains absent |

## Adding or changing a contract

1. Audit the existing specification owners and machine-readable contracts.
2. If the rule is accepted, write the specification change before or alongside
   its implementation.
3. If a concrete requirement exposes an unresolved semantic, compatibility, or
   trust choice, raise it in [`OWNER_QUESTIONS.md`](../../OWNER_QUESTIONS.md).
4. If the behavior is only a candidate, place it in
   [proposals](../proposals/README.md).
5. Link conformance evidence without making generated build output normative.

Specification pages do not contain open-ended design-question sections. They
may identify an exclusion or an unestablished claim. Alternatives and research
remain in design, proposals, or drafts.
