# Phase 7: Trust and Measurement

> Not a step that runs after the others, but a chain woven through every phase: each stage verifies and records the next, anchored in hardware the software cannot forge. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented. The contract is in [Boot, Trust Chain & Recovery](../design/part_5_lifecycle/03_boot_and_trust_chain.md).

## The root of trust is hardware

The chain has to start in something software cannot fake: a tamper-resistant chip such as a Trusted Platform Module ("TPM") or a secure element, or fuses burned into the system-on-chip. Everything above earns trust by being checked against that anchor.

## Measure and verify, stage by stage

At each handoff the current stage does two things before passing control on. It **verifies** the next stage's signature, and it **measures** it: extends a running hash of what has booted into a special hardware register that boot code can only add to, never overwrite. The sequence of measurements is a tamper-evident record of exactly what ran.

- Firmware verifies and measures the kernel image (Secure Boot, [phase 1](01_firmware.md)).
- The kernel verifies the provenance of the system realm and of each component it admits against signed build records, including the realized host-input receipts and graph-wide reproducibility classification ([package system](../design/part_5_lifecycle/00_package_system.md), [distribution](../design/part_7_governance/03_store_and_economic_control.md)).

## Sealing: keys that exist only on a good boot

A key can be **sealed** to the measurements ([secrets & keys](../design/part_1_authority/04_secrets_and_keys.md)): the hardware releases it only if the running measurements match an expected value. Disk and realm keys are sealed this way, and the user realm's key is additionally bound to the user's credential ([phase 6](06_session_and_login.md)). A tampered boot, or the wrong person, leaves the data unreadable rather than merely access-denied.

## Attestation: proving it from outside

The measurement record lets the hardware sign a statement of what booted, which a separate party, a second device or a remote service, can check ([audit & provenance](../design/part_7_governance/01_audit_compliance_provenance.md)). This is the only check that survives a fully compromised machine: nothing it shows you from inside can be trusted, but the hardware signature cannot be forged, so an outside verifier can still tell a genuine boot from a fabricated one.

## Confidential boot

Optionally the whole operating system boots inside a Trusted Execution Environment (a "TEE": a hardware-isolated region whose memory even the host operating system or hypervisor cannot read), so a hostile host cannot inspect or tamper with it, with attestation proving it really runs in a genuine TEE ([Boot, Trust Chain & Recovery](../design/part_5_lifecycle/03_boot_and_trust_chain.md)). The cost is that such a component is deliberately opaque, which works against the system's usual goal of being observable.

## The honest residue

The one stage Cathedral does not write is the pre-kernel firmware ([phase 1](01_firmware.md)), so it stays in the trusted base and is the hardest layer to attest, because it runs beneath anything Cathedral ships.

## Next

[Phase 8: Recovery and failure](08_recovery_and_failure.md).
