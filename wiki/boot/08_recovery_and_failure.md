# Phase 8: Recovery and Failure

> Boot is only robust if every phase has a defined failure path. A beautiful OS that bricks during an update is a dead OS. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented. The contract lives in [Boot, Trust Chain & Recovery](../design/part_5_lifecycle/03_boot_and_trust_chain.md).

## Storage failures

- **Bad or missing superblock.** The superblock is a ring of generation-numbered slots ([phase 4](04_mounting_the_store.md)), so a corrupt slot is skipped and the next-highest valid generation is used. If all slots are bad, boot falls to recovery.
- **Corrupt log or a partial transaction.** Replay stops at the last fully committed transaction and rolls the partial one back. Because the superblock only ever points at a committed root, the worst case is losing the most recent uncommitted work, never a corrupt tree.

## Verification failures

- **Bad signature or a measured-boot mismatch.** A stage that fails to verify the next refuses to continue and enters recovery ([trust & measurement](07_trust_and_measurement.md)).
- **Rollback protection.** Recovery must not let an attacker force a *downgrade* to a known-vulnerable signed version. Legitimate rollback and anti-downgrade are the two sides that have to coexist.

## Update failures: the one that matters most

A half-applied update must be survivable. Because the system realm is immutable and content-addressed ([filesystem](../design/part_4_storage/00_filesystem_as_database.md), [updates & hot swap](../design/part_5_lifecycle/01_updates_and_hot_swap.md)), rolling back is swapping the root pointer in the superblock back to the previous known-good system root. The old system realm was never overwritten, so the rollback is atomic and cheap, not a restore-from-backup. This is the operational form of "upgrade is a designed operation, not a catastrophe."

## The recovery image

A known-good fallback the normal update process cannot corrupt, used when the primary path will not come up. Keeping it un-brickable by the very mechanism it backs up is the central design problem: it has to be updatable enough to stay useful and isolated enough that a bad update cannot take it down with the main system.

## Credential and user-realm failures

If the user realm cannot be unsealed ([phase 6](06_session_and_login.md)), because the credential is lost or the boot is tampered, the machine stays at a pre-session state and offers an account-recovery path. On a multi-tenant device, one tenant's failure or wipe must not destroy another's data ([multi-user & org control](../design/part_6_human_surface/04_multi_user_and_org_control.md)).

## Factory reset

Reset to a clean, attested baseline: drop the user and app realms, restore the system realm to a known-good measured state, and re-enroll. The content-addressed, realm-partitioned store makes "wipe my data but keep the OS" and "reset the OS but keep my data" both expressible, rather than all-or-nothing.
