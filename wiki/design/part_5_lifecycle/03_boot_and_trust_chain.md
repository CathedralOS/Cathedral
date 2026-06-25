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

### The decided mechanism

**The measured chain ends at the static TCB; the TCB then *reports* the dynamic graph.** You cannot measure-boot the live authority graph — it changes every moment — so attestation splits in two. **Static measurement** hashes-and-extends each stage into the hardware root: firmware → a minimal loader shim → the proved-Omega core → the **privileged component manifest** (the bounded TCB from [[kernel_architecture]], not merely "the kernel"). **Dynamic attestation** is then the measured-good core *reporting* the live authority graph it already tracks ([[observability_and_introspection]]), signed with a key sealed to the good boot. Measurement covers the *reporter*; the reporter covers the *runtime* — so attestation reaches the dynamic graph **transitively**: you attest the static reporter and trust its report *because* it booted measured-good. This composes with the build root: the Omega **bootstrap lattice** proves the image is *correct*; **measured boot** proves *which* image is running; together they attest that a *correct* image runs — neither alone does.

**No hardware root → degrade honestly, never fake an attestation.** Measured boot and sealed-to-boot keys are hardware-rooted *by definition*; with no TPM/secure-element/fuses there is no measured boot and no measured unseal. Per the self-attestation principle (a trust root must be *external* to what it attests), a device with no external root **cannot prove anything about itself** — it is **unattestable to others** by construction. The fallback is a software signature-chain (if firmware holds any immutable key) + a **passphrase-derived disk key** (the user supplies the secret at boot instead of the TPM unsealing it): functional, weaker (no anti-tamper, no measured unseal), and the device's attestation surface must *state* "no hardware root, integrity is software-only." A self-report is never dressed up as an attestation.

**Confidential computing is a hardware feature Cathedral *exposes*, not a subsystem it implements.** Confidentiality-from-the-host is realizable only in silicon — software cannot make memory opaque to the OS that owns the page tables; only a hardware TEE (TDX/SEV-SNP/CCA), or a future capability-ISA whose read-barrier even ring-0 cannot bypass, can. So Cathedral does not implement it, and "lose internal observability" is **a hardware fact, not an OS policy choice** — if the silicon enforces opacity, the core cannot see in regardless. The genuinely-Cathedral residue is small: **(1)** expose the TEE as a requestable component property (a conditioning axis like the placement/isolation classes — not new machinery), gated and rare; **(2) attestation-gated capabilities** — a hardware attestation quote becomes a *predicate on a grant* (a secret released only to a requester that proves it runs in a genuine TEE), the capability model absorbing the quote. **External attribution is automatic**: a confidential component still holds its capabilities *through* the OS, so the authority graph still sees that it exists, what it holds, and what it talks to — only its internals are dark. The cost is real (no trace/debug/audit-into; the TCB grows toward the silicon vendor's whole TEE + microcode, a bigger, side-channel-prone root), so it stays opt-in and fenced — and on a custom capability-ISA it dissolves into just another capability.

**Rollback vs. anti-downgrade → a hardware monotonic security floor.** You may roll back **down to a hardware anti-rollback counter** but not below it; the floor advances only on *security-critical* releases (not every update), so ordinary rollback stays free while downgrade to a known-exploitable version is hardware-refused. The recovery image must itself stay ≥ the floor, or it becomes the downgrade hole.

**Un-brickable recovery → the storage A/B + mirror machinery, not new.** The recovery image is a separate immutable **mirrored** placement (the "recovery = the degenerate one-copy mirror" from [[filesystem_as_database]]), updated A/B with an atomic root-flip — a crashed recovery-update leaves the old recovery intact (transactional: power-loss = old-or-new, never corrupt). Boot tries good → fallback → recovery via the firmware boot-attempt counter. It composes the decided transaction/placement machinery rather than adding any.

**The trusted boundary → firmware assumed, a tiny shim trusted, everything above proved.** Firmware sits below Cathedral (the assumed hardware root, not ours). The residual non-Omega trusted boundary is the *smallest possible loader shim* from firmware-handoff to the proved core; the core upward is proved-Omega, built trust-by-checking. The measured chain covers all of it.

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

*(All five resolved by "The decided mechanism" above: HW root assumed-for-attestation / honest-degrade-if-absent; the chain ends at the static TCB with the dynamic graph reported transitively; rollback bounded by a hardware security floor; recovery = the storage A/B-mirror + atomic-flip; confidential components = a hardware feature Cathedral exposes via attestation-gated capabilities, with external attribution automatic.)*

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
