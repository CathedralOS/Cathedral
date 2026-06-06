# Phase 6: Session and Login

> The human arrives. Whether there is a login process, where credentials are stored, and what logging in actually produces. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

After [phase 5](05_components_and_services.md) the machine is running but unattended, showing a login surface with no user authority in play.

## Before login: a sealed user realm

Each user, or each tenant on a shared device, is a **realm** ([multi-user & org control](../design/part_6_human_surface/04_multi_user_and_org_control.md), [filesystem](../design/part_4_storage/00_filesystem_as_database.md)). Before anyone logs in, the user realm exists on disk but is **sealed**: its contents are encrypted and no running principal holds the capability to its root. The user's data is not merely permission-denied, it is unreadable, because the decryption key is not available yet.

## There is a login process

The login surface is run by a trusted **session broker**, a system component drawn on the compositor's trusted path so it cannot be spoofed ([windowing & compositor](../design/part_6_human_surface/00_windowing_and_compositor.md), [permission UX](../design/part_6_human_surface/01_human_permission_ux.md)). It is operating-system code, not a user app, and it holds no user authority itself: it can start an authentication, but it cannot read user data or mint user capabilities on its own.

## Where credentials live

Credentials are **secrets**, kept in the keychain (the operating system's protected secret store) and backed by hardware where the device allows it ([secrets & keys](../design/part_1_authority/04_secrets_and_keys.md)). "Hardware-backed" means held inside a tamper-resistant chip (a Trusted Platform Module, "TPM", or a secure element) that the main CPU cannot read out. They are never plaintext on disk, and the session broker never sees the raw secret. Following the operation-capability principle, the broker is given a narrow `Capability<VerifyCredential>` rather than the credential itself, or the comparison runs inside a secure enclave (an isolated coprocessor) so that a fingerprint or face template never leaves it. Passkeys, biometrics, and passwords are all the same shape: an operation against a sealed secret, not a value passed around.

## Authentication unseals the realm

A correct credential does two things at once. It proves identity, binding the new session to the user's principal ([identity & principals](../design/part_1_authority/02_identity_and_principals.md)). And it releases the user realm's decryption key, which is sealed to the credential plus a measured-good boot ([trust & measurement](07_trust_and_measurement.md)). So logging in is the act that makes the user's data readable: a wrong credential, or a tampered boot, leaves the realm sealed.

## What login produces: a session

Login mints a **session**: a short-lived principal that carries the human's authority for a bounded window and is itself revocable ([identity & principals](../design/part_1_authority/02_identity_and_principals.md), [lifecycle](../design/part_1_authority/01_capability_lifecycle.md)). The session is handed the capability to the user realm's root and the bundle of authority the user delegates to it, and nothing more.

## Launching the user's world

The session then starts the user's first components inside the user realm: the per-session compositor, the home surface or shell, and any autostart apps, each receiving capabilities delegated down from the session ([components](05_components_and_services.md)). The user is now logged in and the machine is usable.

## Lock, logout, switch

Locking or logging out revokes the session and re-seals the user realm, dropping its key from memory. Switching users selects a different realm and runs its own login, so two users' worlds never share ambient authority. The unattended pre-session state is the safe default the machine returns to.

## Next

[Phase 7: Trust and measurement](07_trust_and_measurement.md).
