# Chapter 25: Boot, Trust Chain & Recovery

> The boring, scary stuff: how the system comes up provably from a hardware root of
> trust, and how it survives a bad update without bricking.

## The Legacy Contract

Boot is a chain of mutually-distrustful-but-unverified stages: firmware loads a
bootloader loads a kernel loads userspace, and at each hop "trust" is, classically,
*nothing* — or a Secure Boot signature check that stops at the kernel and never
covers the live component graph. Recovery is an afterthought: a separate partition
that may itself be stale, a factory reset that nukes user state, and a rollback story
that, where it exists, is bolted on by the update tool rather than designed in. The
result is the field's most feared failure: a half-applied update that bricks the
device with no path back.

## What Cathedral Wants

A continuous, **measured** trust chain from a hardware root of trust through
firmware, bootloader, kernel, and every privileged component — each stage verifying
and measuring the next, with the measurements available for remote attestation
([[34_audit_compliance_provenance]]). And, equally first-class: **recovery and
rollback protection.** A beautiful OS that bricks during an update is a dead OS, so
the ability to detect a bad boot, roll back to a known-good system, and recover a
half-applied update is part of the design from the start — not a rescue disk added
later.

## Concerns & Design Space

- **Secure boot & measured boot.** Each stage's signature is verified *and* its
  measurement is extended into a hardware register, so the booted state is both
  authorized and attested.
- **Bootloader & component verification.** Verification does not stop at the kernel;
  the privileged component set is measured too (ties to [[26_kernel_architecture]]).
- **Recovery partition & bricked-update recovery.** A known-good fallback the update
  process cannot corrupt; a half-applied update is detectable and reversible
  ([[23_updates_and_hot_swap]] rollback).
- **Rollback protection.** Prevent an attacker from forcing a *downgrade* to a
  known-vulnerable signed version — the dual of allowing legitimate rollback.
- **Factory reset & device enrollment.** Reset to a clean, attested baseline; enroll
  the device into an org/identity at first boot ([[05_identity_and_principals]]).
- **Disk encryption.** Keys sealed to the trust chain so they unseal only on a
  measured-good boot ([[07_secrets_and_keys]]).
- **Remote attestation & firmware trust.** A relying party can verify what the device
  booted; the firmware layer is itself part of the measured chain.

## Key Questions

- What is the hardware root of trust for the target hardware (TPM, secure element, SoC
  fuses), and what does Cathedral assume vs. require of it?
- Where exactly does the measured chain *end* — at the kernel, at privileged
  components, or at the full live authority graph?
- How do legitimate rollback and anti-downgrade rollback protection coexist without
  one defeating the other?
- What is the minimal recovery image, and how is it kept un-brickable by the very
  update mechanism it backs up?

## Omega Leverage

- **Capabilities as values** make the unseal key, the recovery authority, and the
  enrollment grant first-class held authorities, not ambient firmware powers
  ([[07_secrets_and_keys]]).
- **Authority-flow + boundary reports** describe the boot-time TCB: which privileged
  components the chain admits is an auditable fact, and measured boot can attest the
  same component manifest the build produced.
- **Versioned `data` / signed provenance** ([[22_package_system]]) give the
  monotonic version facts rollback protection needs to reason about downgrades.
- **`effects` ceilings** bound what early-boot components may do before the full
  capability machinery is online.
- Omega does **not** model a hardware root of trust or attestation primitives; the
  sealing/measurement layer is a boundary Cathedral must specify against real silicon.

## Open Questions

- How much of the early-boot path can be proved Omega vs. necessarily a small
  trusted firmware/loader boundary, and how is that residual TCB minimized?
- Can attestation cover the *dynamic* authority graph (which capabilities are held
  post-boot), or only the static booted image?
- What is the trust and recovery story for the firmware layer itself, which sits
  below anything Cathedral controls?

## Related
- [[07_secrets_and_keys]] — hardware root of trust, sealed keys, disk encryption.
- [[23_updates_and_hot_swap]] — rollback and half-applied-update recovery.
- [[26_kernel_architecture]] — what the measured chain must cover.
- [[05_identity_and_principals]] — device identity, enrollment, attestation subject.
- [[34_audit_compliance_provenance]] — measured boot and attestation as evidence.
