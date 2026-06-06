# Phase 7: Trust and Measurement

> Not a step that happens after the others, but the security spine running *through* every phase: each stage verifies and measures the next, anchored in hardware. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented. The contract lives in [Boot, Trust Chain & Recovery](../design/part_5_lifecycle/03_boot_and_trust_chain.md).

## The root of trust is hardware

The chain has to be anchored in something the software cannot forge: a TPM, a secure element, or SoC fuses. Everything above earns trust by being measured and verified against that anchor.

## Measure and verify, stage by stage

Each stage checks the next before handing off, and records what it ran:

- Firmware verifies the kernel image's signature before loading it (Secure Boot, [phase 1](01_firmware.md)).
- The kernel verifies the provenance of the system realm and the components it admits, against signed, reproducible build manifests ([package system](../design/part_5_lifecycle/00_package_system.md), [distribution](../design/part_7_governance/03_store_and_economic_control.md)).
- Each stage extends a **measurement** into a hardware register, so the booted state is both authorized and recorded.

## Sealing: keys that only exist on a good boot

Disk and realm encryption keys are **sealed** to a measured-good boot ([secrets & keys](../design/part_1_authority/04_secrets_and_keys.md)): they unseal only if the measurements match the expected state. The user realm's keys are additionally bound to the user's credential ([phase 6](06_session_and_login.md)), so a tampered boot, or the wrong person, leaves the data unreadable rather than merely access-denied.

## Attestation: proving it from outside

The measurement log lets a relying party (a second device, a remote service) verify what the machine actually booted ([audit & provenance](../design/part_7_governance/01_audit_compliance_provenance.md)). This is the only check that survives a fully compromised host: you cannot trust anything the machine draws from inside, but the hardware can sign a measurement that an outside verifier checks. It is the way out of a fabricated world.

## Confidential boot

Optionally the whole OS boots inside a Trusted Execution Environment so a hostile host (a cloud operator, a hypervisor) cannot read or tamper with it, with attestation proving it runs in a genuine TEE ([Boot, Trust Chain & Recovery](../design/part_5_lifecycle/03_boot_and_trust_chain.md)). The trade is that a confidential component is deliberately opaque, which fights the OS's observability thesis.

## The honest residue

The one stage Cathedral does not author is the pre-kernel firmware ([phase 1](01_firmware.md)), so it sits in the trusted base and is the hardest layer to attest, because it is below anything we ship. Shrinking or replacing it is the long-term goal noted there.

## Next

[Phase 8: Recovery and failure](08_recovery_and_failure.md) — what happens when any of this goes wrong.
