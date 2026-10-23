# Program isolation

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-23 |

## Hardware-separated programs

The owner accepted on 2026-10-23 that independently isolated programs use
hardware-separated address spaces on the current MMU-based targets. This applies
to applications, matrices and platform providers alike. Being shipped with the
OS, written in Omega or checked by a compiler does not waive that boundary.
The privileged kernel remains the trusted implementation of those boundaries;
this does not require separate address spaces for its internal modules.

Single-address-space isolation based on proof-carrying code is removed from the
current architecture and roadmap. The owner judged that approach insufficiently
ergonomic and viable for Cathedral. It is not an optional admission route awaiting
a proof threshold, and is not an aspirational requirement for the Omega port.

Explicitly authorized shared mappings remain supported between otherwise
separate programs. Their permissions and lifetime must be enforced at the actual
hardware granularity. Checked shared-memory protocols and trusted provider
implementations still have a role, but do not grant arbitrary native writers
sub-page isolation or justify merging independent protection domains.

## Future capability hardware

Reconsidering a single address space requires CHERI support in the owner's
current direction. This is a future hardware-dependent design, not an accepted
CHERI port or a claim that CHERI alone establishes every required guarantee.
Its compartment, authority, scheduling, teardown and interoperability contracts
would require separate specification and evidence before admission.

## Evidence and limits

The [Rust machine boundary](../../../source-rs/kernel/arch/README.md) provides
x86-64 and AArch64 address-space and execution mechanisms for the reference lab.
The current task model uses hardware isolation. This acceptance does not freeze
its syscall ABI or certify every implementation detail.

Separate address spaces do not settle kernel page-table placement, translation
cache optimizations such as PCID, side-channel policy or their validation scope.
These must preserve the isolation contract and have their own evidence. The
[matrix lifetime](components_and_matrices.md#matrix-failure-and-subtree-lifetime)
and [execution budget](../resources/execution_budgets.md) contracts still apply.
