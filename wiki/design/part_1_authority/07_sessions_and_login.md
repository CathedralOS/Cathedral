# Chapter 07: Sessions & Login

> How a human becomes an authenticated principal holding a session, where credentials live, and how authentication unseals the user's realm.

## The Legacy Model

On a legacy system, login produces an ambient session: authenticate once and your processes run as your `uid` with everything that `uid` can do, with no further structure. Credentials are checked against a shadow file or a directory service, and "logged in" is a permission bundle attached to a process tree. There is no first-class session object to scope, lease, or revoke, so logout kills processes rather than ending a bounded grant. And a user's data is readable the moment the account exists, gated by file permissions rather than by whether the user has actually proven who they are; the bytes were never sealed.

## The Cathedral Model

Login is an authority-minting event. Authentication binds a new **session**, a short-lived principal ([[identity_and_principals]]), to the user's principal and produces a bounded, revocable bundle of capabilities, chiefly the root capability to the user's realm ([[filesystem_as_database]], [[multi_user_and_org_control]]). Before login the user realm is sealed: encrypted, with no principal holding its root, so the data is unreadable rather than merely access-denied. A correct credential releases the realm's key, which is sealed to the credential plus a measured-good boot ([[secrets_and_keys]], [[boot_and_trust_chain]]), and mints the session. Logout or lock revokes the session and re-seals the realm.

The login surface is a trusted broker on the compositor's trusted path ([[windowing_and_compositor]], [[human_permission_ux]]). It holds no user authority itself: it can start an authentication but cannot read user data or mint user capabilities on its own. Credentials are operation-capabilities, not values, so the broker verifies through a narrow `Capability<VerifyCredential>` or a secure enclave performs the match, and a password hash, passkey, or biometric template never passes through it ([[secrets_and_keys]]).

## Concerns & Design Space

- **The session as a principal.** A leased ([[capability_lifecycle]]) principal carrying a human's delegated authority for a bounded window; everything the user's apps hold is delegated down from it.
- **Sealing the user realm.** The realm's key is sealed to the credential and the boot measurement, so login is the act that makes the data readable ([[secrets_and_keys]], [[boot_and_trust_chain]]).
- **On-demand and nested sealing.** Sealing is not only the whole user realm at login; any subtree can be its own sealed realm with its own credential, unsealed on demand and re-sealed when done. The unseal prompt runs on the trusted path, so the requesting app or shell triggers the unlock but never sees the credential or the key ([[secrets_and_keys]], [[filesystem_as_database]]).
- **Credentials as operations.** Passkeys, biometrics, and passwords are operations against a sealed secret; the broker never sees raw credential material.
- **A broker with no user authority.** It can initiate authentication and mint a session only by invoking the authority machinery, never by possessing the user's capabilities.
- **Multi-user and switching.** Each user or tenant is a realm; switching runs a separate login; two sessions never share ambient authority ([[multi_user_and_org_control]]).
- **Lock, logout, timeout.** Revoke the session and drop the realm key from memory; an idle session is a lease that can expire.
- **Remote and federated login.** Authenticating against an organization identity or a remote principal to the same standard as a local one ([[identity_and_principals]], [[distributed_boundary]]).
- **Recovery.** A lost credential or a tampered boot leaves the realm sealed; the account-recovery path must not become a backdoor ([[boot_and_trust_chain]]).
- **Zero value.** A zeroed session is the unauthenticated pre-login principal (shape 1 in [[omega_substrate]]): a valid principal holding no capabilities with the realm still sealed, so the not-yet-logged-in state is the default and an uninitialized session can read nothing rather than defaulting open.

## Key Questions

- What exactly is in a session's capability bundle at mint time, and how is it derived from the user's standing authority?
- How is the realm key sealed and released, and what is the fallback when the sealing hardware is absent or fails?
- Is a session a distinct principal kind, or a lease over the user principal?
- What recovery path restores a lost credential without weakening the sealing guarantee?

## Omega Leverage

- A session is a **principal** ([[identity_and_principals]]) holding **capabilities** ([[capability_model]]) under a **lease** ([[capability_lifecycle]]); no new machinery.
- Credentials as **operation-capabilities** (`Capability<VerifyCredential>`) reuse the secrets model ([[secrets_and_keys]]).
- Sealing binds to the boot **measurement**, the trust-chain artifact ([[boot_and_trust_chain]]).
- Omega does not define credential or attestation protocols; those are runtime and provider structure Cathedral specifies.

## Open Questions

- Can "the broker never holds user authority" be proven structurally, or is it a runtime discipline?
- How are passwordless and continuous authentication (presence, proximity) modeled as evolving session leases?

## Related
- [[identity_and_principals]] — a session is a principal.
- [[secrets_and_keys]] — credentials and realm-key sealing.
- [[capability_lifecycle]] — the session as a lease.
- [[multi_user_and_org_control]] — tenants as realms, switching.
- [[filesystem_as_database]] — the user realm the session unseals.
- [[boot_and_trust_chain]] — sealing to a measured boot.
- [[human_permission_ux]] — the trusted login surface.
