# Phase 8: Recovery and Failure

> Boot is only robust if every phase has a defined failure path. An operating system that bricks during an update is a dead one. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented. The contract is in [Boot, Trust Chain & Recovery](../design/part_5_lifecycle/03_boot_and_trust_chain.md).

## Storage failures

- **Bad or missing superblock.** The superblock is a ring of generation-numbered slots ([phase 4](04_mounting_the_store.md)), so a corrupt slot is skipped and the next-highest valid generation is used. If every slot is bad, boot falls to recovery.
- **Corrupt log or half-written transaction.** Replay stops at the last fully committed transaction and discards the partial one. Because the superblock only ever points at a committed root, the worst case is losing the most recent uncommitted work, never a corrupt tree.

## Verification failures

- **Bad signature or a measurement mismatch.** A stage that cannot verify the next refuses to continue and enters recovery ([trust & measurement](07_trust_and_measurement.md)).
- **Downgrade protection.** Recovery must not let an attacker force a return to an older, known-vulnerable but still validly signed version. Allowing a legitimate rollback while forbidding a malicious downgrade is the tension to resolve.

## Update failures: the one that matters most

A half-applied update has to be survivable. Because the system realm is immutable and content-addressed ([filesystem](../design/part_4_storage/00_filesystem_as_database.md), [updates & hot swap](../design/part_5_lifecycle/01_updates_and_hot_swap.md)), rolling back is just pointing the superblock's `root_ref` at the previous known-good system root. The old system realm was never overwritten, so the rollback is atomic and cheap, not a restore from backup. This is the operational form of treating an upgrade as a designed operation rather than a leap of faith.

## The recovery image

A known-good fallback the normal update process cannot corrupt, used when the primary path will not come up. The hard part is keeping it un-brickable by the very mechanism it backs up: it has to be updatable enough to stay useful, yet isolated enough that a bad update cannot take it down along with the main system. A fuller version of the same idea is mirroring the system realm across every enrolled drive (a placement class, [filesystem](../design/part_4_storage/00_filesystem_as_database.md)), so any drive boots and the recovery image becomes the minimal one-copy case.

## Credential and user-realm failures

If the user realm cannot be unsealed ([phase 6](06_session_and_login.md)), because the credential is lost or the boot is tampered, the machine stays at the pre-session state and offers an account-recovery path. On a shared device, one tenant's failure or wipe must not destroy another tenant's data ([multi-user & org control](../design/part_6_human_surface/04_multi_user_and_org_control.md)).

## Factory reset

Reset to a clean, attested baseline: drop the user and app realms, restore the system realm to a known-good measured state, and re-enroll. Because the store is content-addressed and partitioned into realms, "wipe my data but keep the operating system" and "reset the operating system but keep my data" are both expressible, instead of all-or-nothing.
