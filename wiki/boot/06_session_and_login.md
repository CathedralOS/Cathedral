# Phase 6: Session and Login

> The human arrives. Is there a login process? Where are credentials stored? What does logging in actually produce? Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

After [phase 5](05_components_and_services.md) the machine is running but unattended, sitting at a login surface with no user authority anywhere in play.

## Before login: a sealed user realm

Each user (or tenant) is a **realm** ([multi-user & org control](../design/part_6_human_surface/04_multi_user_and_org_control.md), [filesystem](../design/part_4_storage/00_filesystem_as_database.md)). Before anyone logs in, the user realm exists on disk but is **sealed**: its bodies are encrypted and no running principal holds its root capability. So the user's data is not merely permission-denied, it is unreadable, because the keys are not available yet.

## Yes, there is a login process

The login surface is run by a trusted **session broker**, a system component on the compositor's trusted path ([windowing & compositor](../design/part_6_human_surface/00_windowing_and_compositor.md), [permission UX](../design/part_6_human_surface/01_human_permission_ux.md)). It is OS code, not a user app, and crucially it holds **no user authority itself**: it can start an authentication, but it cannot read user data or mint user capabilities on its own.

## Where credentials live

Credentials are **secrets**, stored in the keychain / secret store and hardware-backed where the device allows ([secrets & keys](../design/part_1_authority/04_secrets_and_keys.md)), sealed to the secure element or TPM. They are never plaintext on disk, and the session broker never sees the raw secret. Following the operation-capability principle, the broker is given a narrow `Capability<VerifyCredential>` rather than the credential bytes, or the match runs entirely inside a secure enclave (a fingerprint or face template never leaves it). Passkeys, biometrics, and passwords are all this same shape: an operation against a sealed secret, not a value handed around.

## Authentication unseals the realm

A correct credential does two things at once. It proves identity, binding a session to the user's principal ([identity & principals](../design/part_1_authority/02_identity_and_principals.md)). And it **unseals the user realm's keys**, which are sealed to the credential plus a measured-good boot ([trust & measurement](07_trust_and_measurement.md)). So logging in is literally the act that makes the user's data readable; a wrong credential, or a tampered boot, leaves the realm sealed.

## What login produces: a session

Login mints a **session**: a short-lived principal that carries the human's authority for a bounded window, itself leasable and revocable ([identity & principals](../design/part_1_authority/02_identity_and_principals.md), [lifecycle](../design/part_1_authority/01_capability_lifecycle.md)). This is an authority mint at the largest scale ([permission UX](../design/part_6_human_surface/01_human_permission_ux.md)): the session is handed the root capability to the user realm and the bundle of authority the user delegates to this session, and nothing more.

## Launching the user's world

The session then spawns the user's first components *inside the user realm*: the per-session compositor, the shell or home surface, and any autostart apps, each receiving capabilities delegated down from the session ([components](05_components_and_services.md)). The user is now logged in and the system is usable.

## Lock, logout, switch

Locking or logging out revokes the session (ending its leased authority) and re-seals the user realm, dropping its keys from memory. Switching users selects a different realm and runs its own login, so two users' worlds never share ambient authority. On a shared device, the pre-session state is the safe default the machine returns to.

## Next

[Phase 7: Trust and measurement](07_trust_and_measurement.md) — the security spine that ran through every phase above, including the sealing this phase depended on.
