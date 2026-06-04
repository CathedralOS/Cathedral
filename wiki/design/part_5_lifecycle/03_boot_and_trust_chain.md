# Chapter 03: Boot, Trust Chain & Recovery

> The boring, scary stuff: how the system comes up provably from a hardware root of trust, and how it survives a bad update without bricking.

## The Legacy Model

Boot is a chain of mutually-distrustful-but-unverified stages: firmware loads a bootloader loads a kernel loads userspace, and at each hop "trust" is, classically, *nothing* — or a Secure Boot signature check that stops at the kernel and never covers the live component graph. Recovery is an afterthought: a separate partition that may itself be stale, a factory reset that nukes user state, and a rollback story that, where it exists, is bolted on by the update tool rather than designed in. The result is the field's most feared failure: a half-applied update that bricks the device with no path back.

## The Cathedral Model

A continuous, **measured** trust chain from a hardware root of trust through firmware, bootloader, kernel, and every privileged component — each stage verifying and measuring the next, with the measurements available for remote attestation ([[audit_compliance_provenance]]). And, equally first-class: **recovery and rollback protection.** A beautiful OS that bricks during an update is a dead OS, so the ability to detect a bad boot, roll back to a known-good system, and recover a half-applied update is part of the design from the start.

### Confidential computing: distrusting the host

Measured boot above proves to a relying party *what the device booted*, and defends against unauthorized code running on your hardware. Confidential computing flips the threat model: it defends a workload against the machine it runs on. The owner of the hardware (a cloud operator, a hypervisor, an administrator, or someone with physical access and a DRAM probe) is treated as the adversary. The mechanism is a Trusted Execution Environment with encrypted, integrity-protected memory: Intel TDX, AMD SEV-SNP, and ARM CCA Realms at VM granularity, with attestation proving the workload runs in a genuine TEE on genuine silicon.

Two ways this matters for Cathedral:

- **Cathedral as the guest.** The whole OS can run inside a confidential VM so a hostile cloud host cannot read or tamper with it. The trust chain then attests not just "this image booted" but "this image booted in a TEE the host cannot see into" ([[audit_compliance_provenance]]).
- **Cathedral as the host.** A component can be given memory that the rest of the system, *including privileged code*, cannot read. This adds a conditioning axis to the capability model: a capability can be **attestation-gated**, granted only to a requester that proves it runs in a measured, genuine TEE. A remote service can hand a secret to a component because the component proved *where and how it runs*, not because it presented a password.

The honest tensions, which belong in the doc rather than glossed over:

- **It grows the TCB toward the silicon vendor.** You are now trusting the CPU vendor's TEE implementation and microcode, and TEEs have a long history of side-channel breaks. That confidentiality rests on the vendor's silicon being sound.
- **It is in direct tension with this OS's thesis.** Cathedral's pitch is observable, introspectable, queryable behavior. A confidential component is deliberately *opaque*: you cannot trace, debug, or audit its internals from outside ([[observability_and_introspection]], [[debugging_and_tracing]]). The design must say explicitly which components may be confidential and what observability the user or operator knowingly gives up to gain it. That trade is a policy, not a default.

## Concerns & Design Space

- **Secure boot & measured boot.** Each stage's signature is verified *and* its measurement is extended into a hardware register, so the booted state is both authorized and attested.
- **Bootloader & component verification.** Verification does not stop at the kernel; the privileged component set is measured too (ties to [[kernel_architecture]]).
- **Recovery partition & bricked-update recovery.** A known-good fallback the update process cannot corrupt; a half-applied update is detectable and reversible ([[updates_and_hot_swap]] rollback).
- **Rollback protection.** Prevent an attacker from forcing a *downgrade* to a known-vulnerable signed version — the dual of allowing legitimate rollback.
- **Factory reset & device enrollment.** Reset to a clean, attested baseline; enroll the device into an org/identity at first boot ([[identity_and_principals]]).
- **Disk encryption.** Keys sealed to the trust chain so they unseal only on a measured-good boot ([[secrets_and_keys]]).
- **Remote attestation & firmware trust.** A relying party can verify what the device booted; the firmware layer is itself part of the measured chain.
- **Confidential computing.** Encrypted-memory TEEs (TDX, SEV-SNP, CCA) let a workload distrust its host; capabilities can be attestation-gated; confidential components are deliberately opaque, which trades directly against observability ([[observability_and_introspection]]).

## Key Questions

- What is the hardware root of trust for the target hardware (TPM, secure element, SoC fuses), and what does Cathedral assume vs. require of it?
- Where exactly does the measured chain *end* — at the kernel, at privileged components, or at the full live authority graph?
- How do legitimate rollback and anti-downgrade rollback protection coexist without one defeating the other?
- What is the minimal recovery image, and how is it kept un-brickable by the very update mechanism it backs up?
- Which components, if any, may be confidential (host-opaque), and what observability and debuggability does the system knowingly give up for them ([[observability_and_introspection]], [[debugging_and_tracing]])?

## Omega Leverage

- **Capabilities as values** make the unseal key, the recovery authority, and the enrollment grant first-class held authorities, not ambient firmware powers ([[secrets_and_keys]]).
- **Authority-flow + boundary reports** describe the boot-time TCB: which privileged components the chain admits is an auditable fact, and measured boot can attest the same component manifest the build produced.
- **Versioned `data` / signed provenance** ([[package_system]]) give the monotonic version facts rollback protection needs to reason about downgrades.
- **`effects` ceilings** bound what early-boot components may do before the full capability machinery is online.
- Omega does **not** model a hardware root of trust or attestation primitives; the sealing/measurement layer is a boundary Cathedral must specify against real silicon.

## Open Questions

- How much of the early-boot path can be proved Omega vs. necessarily a small trusted firmware/loader boundary, and how is that residual TCB minimized?
- Can attestation cover the *dynamic* authority graph (which capabilities are held post-boot), or only the static booted image?
- What is the trust and recovery story for the firmware layer itself, which sits below anything Cathedral controls?
- Is running Cathedral inside a confidential VM, or hosting confidential components, worth the protection against a hostile host given the TEE side-channel track record and the observability it costs?

## Related
- [[secrets_and_keys]] — hardware root of trust, sealed keys, disk encryption.
- [[updates_and_hot_swap]] — rollback and half-applied-update recovery.
- [[kernel_architecture]] — what the measured chain must cover.
- [[identity_and_principals]] — device identity, enrollment, attestation subject.
- [[audit_compliance_provenance]] — measured boot and attestation as evidence.
