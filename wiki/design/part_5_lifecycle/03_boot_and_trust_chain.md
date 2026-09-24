# Chapter 03: Boot, Trust Chain & Recovery

> How the system comes up provably from a hardware root of trust, and how it survives a bad update without bricking.

## The Legacy Model

Boot is a chain of stages that distrust each other but never verify each other. Firmware loads a bootloader, which loads a kernel, which loads userspace, and at each hop the trust is classically nothing, or a Secure Boot signature check that stops at the kernel and never covers the live component graph. Recovery is an afterthought: a separate partition that may itself be stale, a factory reset that wipes user state, and a rollback story that, where it exists, is bolted on by the update tool rather than designed in. The result is the field's most feared failure, a half-applied update that bricks the device with no path back.

## The Cathedral Model

Cathedral boots over a continuous **measured** trust chain from a hardware root of trust through firmware, bootloader, kernel, and every privileged component. Each stage verifies and measures the next, and the measurements are available for remote attestation ([[audit_compliance_provenance]]). Recovery and rollback protection carry equal weight. An OS that bricks during an update is a dead OS, so detecting a bad boot, rolling back to a known-good system, and recovering a half-applied update are part of the design from the start.

### Confidential computing: distrusting the host

Confidential computing defends a workload against the machine it runs on. Measured boot proves to a relying party what the device booted and defends against unauthorized code running on your hardware. Confidential computing flips that threat model: the owner of the hardware (a cloud operator, a hypervisor, an administrator, or someone with physical access and a DRAM probe) is treated as the adversary. The mechanism is a Trusted Execution Environment with encrypted, integrity-protected memory. Intel TDX, AMD SEV-SNP, and ARM CCA Realms provide it at VM granularity, with attestation proving the workload runs in a genuine TEE on genuine silicon.

It matters for Cathedral in two ways:

- **Cathedral as the guest.** The whole OS can run inside a confidential VM so a hostile cloud host cannot read or tamper with it. The trust chain then attests not just "this image booted" but "this image booted in a TEE the host cannot see into" ([[audit_compliance_provenance]]).
- **Cathedral as the host.** A component can be given memory that the rest of the system, including privileged code, cannot read. This adds a conditioning axis to the capability model: a capability can be **attestation-gated**, granted only to a requester that proves it runs in a measured, genuine TEE. A remote service can hand a secret to a component because the component proved where and how it runs, not because it presented a password.

Two tensions come with it:

- **It grows the TCB toward the silicon vendor.** You are trusting the CPU vendor's TEE implementation and microcode, and TEEs have a long history of side-channel breaks. That confidentiality rests on the vendor's silicon being sound.
- **It cuts against the OS's thesis.** Cathedral's pitch is observable, introspectable, queryable behavior. A confidential component is opaque by design: you cannot trace, debug, or audit its internals from outside ([[observability_and_introspection]], [[debugging_and_tracing]]). The design must name which components may be confidential and what observability the user or operator knowingly gives up to gain it. That trade is a policy, not a default.

### The decided mechanism

The measured chain ends at the static TCB, and the TCB then reports the dynamic graph. You cannot measure-boot the live authority graph, because it changes every moment, so attestation splits in two. **Static measurement** hashes and extends each stage into the hardware root: firmware, then a minimal loader shim, then the proved-Omega core, then the **privileged component manifest**, which is the bounded TCB from [[kernel_architecture]] rather than merely "the kernel". **Dynamic attestation** is the measured-good core reporting the live authority graph it already tracks ([[observability_and_introspection]]), signed with a key sealed to the good boot. Measurement covers the reporter and the reporter covers the runtime, so attestation reaches the dynamic graph transitively: you attest the static reporter and trust its report because it booted measured-good. This composes with the build root. The Omega bootstrap lattice proves the image is correct. Measured boot proves which image is running. Together they attest that a correct image runs, and neither does alone.

Without a hardware root the device degrades and never fakes an attestation. Measured boot and sealed-to-boot keys are hardware-rooted by definition. With no TPM, secure element, or fuses there is no measured boot and no measured unseal. A trust root must be external to what it attests, so a device with no external root cannot prove anything about itself. It is unattestable to others by construction. The fallback is a software signature chain, if firmware holds any immutable key, plus a passphrase-derived disk key, where the user supplies the secret at boot instead of the TPM unsealing it. That is functional but weaker: no anti-tamper and no measured unseal. The device's attestation surface must state "no hardware root, integrity is software-only." A self-report is never dressed up as an attestation.

Confidential computing is a hardware feature Cathedral exposes, not a subsystem it implements. Confidentiality from the host is realizable only in silicon. Software cannot make memory opaque to the OS that owns the page tables. Only a hardware TEE (TDX, SEV-SNP, CCA), or a future capability-ISA whose read barrier even ring 0 cannot bypass, can. Cathedral therefore does not implement it, and losing internal observability is a hardware fact rather than an OS policy choice: if the silicon enforces opacity, the core cannot see in regardless. The Cathedral-specific residue is small. First, expose the TEE as a requestable component property, a conditioning axis like the placement and isolation classes rather than new machinery, gated and rare. Second, attestation-gated capabilities: a hardware attestation quote becomes a predicate on a grant, such as a secret released only to a requester that proves it runs in a genuine TEE, and the capability model absorbs the quote. External attribution is automatic. A confidential component still holds its capabilities through the OS, so the authority graph still sees that it exists, what it holds, and what it talks to. Only its internals are dark. The cost is real: no trace, debug, or audit into the component, and the TCB grows toward the silicon vendor's whole TEE plus microcode, a bigger and side-channel-prone root. It therefore stays opt-in and fenced. On a custom capability-ISA it dissolves into just another capability.

Rollback and anti-downgrade are reconciled by a hardware monotonic security floor. You may roll back down to a hardware anti-rollback counter but not below it. The floor advances only on security-critical releases, not every update, so ordinary rollback stays free while downgrade to a known-exploitable version is refused by hardware. The recovery image must itself stay at or above the floor, or it becomes the downgrade hole.

Un-brickable recovery reuses the storage A/B and mirror machinery rather than adding any. The recovery image is a separate immutable mirrored placement, the "recovery is the degenerate one-copy mirror" case from [[filesystem_as_database]], updated A/B with an atomic root flip. A crashed recovery update leaves the old recovery intact, because the update is transactional: a power loss yields old or new, never corrupt. Boot tries good, then fallback, then recovery, driven by the firmware boot-attempt counter. This composes the transaction and placement machinery already in place.

The trusted boundary is firmware assumed, a tiny shim trusted, and everything above proved. Firmware sits below Cathedral as the assumed hardware root, not ours. The residual non-Omega trusted boundary is the smallest possible loader shim from firmware handoff to the proved core. From the core upward the system is proved Omega, built trust-by-checking. The measured chain covers all of it.

## Concerns & Design Space

- **Secure boot & measured boot.** Each stage's signature is verified and its measurement is extended into a hardware register, so the booted state is both authorized and attested.
- **Bootloader & component verification.** Verification does not stop at the kernel. The privileged component set is measured too (ties to [[kernel_architecture]]).
- **Recovery partition & bricked-update recovery.** A known-good fallback the update process cannot corrupt. A half-applied update is detectable and reversible ([[updates_and_hot_swap]] rollback).
- **Rollback protection.** Prevent an attacker from forcing a downgrade to a known-vulnerable signed version, which is the dual of allowing legitimate rollback.
- **Factory reset & device enrollment.** Reset to a clean, attested baseline. Enroll the device into an org/identity at first boot ([[identity_and_principals]]).
- **Disk encryption.** Keys sealed to the trust chain so they unseal only on a measured-good boot ([[secrets_and_keys]]).
- **Remote attestation & firmware trust.** A relying party can verify what the device booted. The firmware layer is itself part of the measured chain.
- **Confidential computing.** Encrypted-memory TEEs (TDX, SEV-SNP, CCA) let a workload distrust its host. Capabilities can be attestation-gated. Confidential components are opaque by design, which trades directly against observability ([[observability_and_introspection]]).

## Key Questions

- What is the hardware root of trust for the target hardware (TPM, secure element, SoC fuses), and what does Cathedral assume versus require of it? Cathedral assumes a hardware root for attestation and degrades to software-only integrity without one.
- Where does the measured chain end: at the kernel, at privileged components, or at the full live authority graph? At the static TCB, with the dynamic graph reported transitively by the measured-good core.
- How do legitimate rollback and anti-downgrade rollback protection coexist without one defeating the other? Through a hardware monotonic security floor that advances only on security-critical releases.
- What is the minimal recovery image, and how is it kept un-brickable by the very update mechanism it backs up? It is the storage A/B mirror placement with an atomic root flip.
- Which components, if any, may be confidential (host-opaque), and what observability and debuggability does the system knowingly give up for them ([[observability_and_introspection]], [[debugging_and_tracing]])? Confidentiality is a hardware feature exposed through attestation-gated capabilities with external attribution automatic; the per-component policy is not yet fixed.

## Omega Leverage

- **Capabilities as values** make the unseal key, the recovery authority, and the enrollment grant held authorities, not ambient firmware powers ([[secrets_and_keys]]).
- **Authority-flow + boundary reports** describe the boot-time TCB: which privileged components the chain admits is an auditable fact, and measured boot can attest the same component manifest the build produced.
- **Normalized schema/artifact identities + signed provenance** ([[package_system]]) give rollback protection permanent facts to compare without treating compatibility as identity.
- **`reaches` ceilings** bound what early-boot components may do before the full capability machinery is online.
- **Constraint-bearing materialization + checked assembly** cover installation of admitted loader artifacts and AP trampolines. Eligibility is established before installation and cannot be manufactured from bytes. This is distinct from later component replacement.
- **External-root reports** make boot, exception, interrupt, and AP entries declared analysis roots even though no ordinary Omega caller reaches them.
- Omega does not model a hardware root of trust or attestation primitives. The sealing/measurement layer is a boundary Cathedral must specify against real silicon.

## Open Questions

- How much of the early-boot path can be proved Omega versus necessarily a small trusted firmware/loader boundary, and how is that residual TCB minimized?
- What is the trust and recovery story for the firmware layer itself, which sits below anything Cathedral controls?
- Is running Cathedral inside a confidential VM, or hosting confidential components, worth the protection against a hostile host given the TEE side-channel track record and the observability it costs?

## Related
- [[secrets_and_keys]] — hardware root of trust, sealed keys, disk encryption.
- [[updates_and_hot_swap]] — rollback and half-applied-update recovery.
- [[kernel_architecture]] — what the measured chain must cover.
- [[identity_and_principals]] — device identity, enrollment, attestation subject.
- [[audit_compliance_provenance]] — measured boot and attestation as evidence.
