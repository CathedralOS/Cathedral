# Phase 6: Session and Login

> The point where a human enters: whether there is a login process, where credentials are stored, and what logging in produces. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented. The design contract is in [Sessions & Login](../design/part_1_authority/07_sessions_and_login.md).

After [phase 5](05_components_and_services.md) the machine is running but unattended, showing a login surface with no user authority in play.

## Before login: a sealed user realm

Each user, or each tenant on a shared device, is a **realm** ([multi-user & org control](../design/part_6_human_surface/04_multi_user_and_org_control.md), [filesystem](../design/part_4_storage/00_filesystem_as_database.md)). Before anyone logs in, the user realm exists on disk but is **sealed**: its contents are encrypted and no running principal holds the capability to its root. The user's data is not merely permission-denied, it is unreadable, because the decryption key is not available yet.

## There is a login process

The login surface is run by a trusted **session broker**, a system component drawn on the compositor's trusted path so it cannot be spoofed ([windowing & compositor](../design/part_6_human_surface/00_windowing_and_compositor.md), [permission UX](../design/part_6_human_surface/01_human_permission_ux.md)). It is operating-system code, not a user app, and it holds no user authority itself: it can start an authentication, but it cannot read user data or mint user capabilities on its own.

## Where credentials live

Credentials are **secrets**, kept in the keychain (the operating system's protected secret store) and backed by hardware where the device allows it ([secrets & keys](../design/part_1_authority/04_secrets_and_keys.md)). "Hardware-backed" means held inside a tamper-resistant chip (a Trusted Platform Module, "TPM", or a secure element) that the main CPU cannot read out. They are never plaintext on disk, and the session broker never sees the raw secret. Following the operation-capability principle, the broker is given a narrow `Capability<VerifyCredential>` rather than the credential itself, or the comparison runs inside a secure enclave (an isolated coprocessor) so that a fingerprint or face template never leaves it. Passkeys, biometrics, and passwords are all the same shape: an operation against a sealed secret, not a value passed around.

## Authentication unseals the realm

For an initially sealed realm, authentication may authorize the storage/crypto
provider to unlock access and establish a session. The provider retains key
custody; the program receives authorized storage operations, not the realm key.
The measured-boot/key mechanism remains part of the
[trust design](07_trust_and_measurement.md). Re-authenticating an existing locked
session is not necessarily a new storage unlock.

## What login produces: a session

Login mints a **session**: a short-lived principal that carries the human's authority for a bounded window and is itself revocable ([identity & principals](../design/part_1_authority/02_identity_and_principals.md), [lifecycle](../design/part_1_authority/01_capability_lifecycle.md)). The session is handed the capability to the user realm's root and the bundle of authority the user delegates to it, and nothing more.

## Launching the user's world

The session then starts the user's first components inside the user realm: the per-session compositor, the home surface or shell, and any autostart apps, each receiving capabilities delegated down from the session ([components](05_components_and_services.md)). The user is logged in and the machine is usable.

## Lock, logout, switch

The accepted [session-lock contract](../spec/human_surface/compositor_and_seat.md#session-locking)
gates interactive access and routes unlock interaction to the authorized interface.
It does not automatically revoke storage grants, destroy keys or re-seal a realm.
Session policy separately chooses whether programs continue, throttle or suspend.

Suspending any program gates its owned descendants under the general
[execution rule](../spec/resources/execution_budgets.md#owner-controlled-subtree-suspension).
Resume preserves budgets, grants and independent suspension gates. A Matrix is
not a special kernel category, and independent providers need not stop with it.

Logout ends a session through its explicit lifecycle policy; deliberate storage
sealing is a separate provider operation, not a consequence inferred from a UI
transition. Switching sessions changes authorized seat routing without granting
one session access to another's realm. The initial sealed pre-login condition is
not a claim that every later lock or switch recreates that cryptographic state.

## Next

[Phase 7: Trust and measurement](07_trust_and_measurement.md).
