# Chapter 07: Sessions & Login

> How a human becomes an authenticated principal holding a session, where credentials live, and how authentication unseals the user's realm.

## The Legacy Model

On a legacy system, login produces an ambient session: authenticate once and your processes run as your `uid` with everything that `uid` can do, with no further structure. Credentials are checked against a shadow file or a directory service, and "logged in" is a permission bundle attached to a process tree. There is no session object to scope, lease, or revoke, so logout kills processes rather than ending a bounded grant. A user's data is readable the moment the account exists, gated by file permissions rather than by whether the user has proven who they are. The bytes were never sealed.

## The Cathedral Model

Login is an authority-minting event over **sealed** data. Authentication releases the key to the user's realm and binds a bounded, revocable session to the user's principal. Before login the realm is sealed: encrypted, with no principal holding its root, so the data is unreadable, not merely access-denied. A correct credential releases the realm's key, which is sealed to the credential plus a measured-good boot, and mints the session. Logout or lock revokes the session and re-seals the realm.

"User" is not an OS concept. The OS knows only three things: **Matrices** (sealed realms), **seats** (a display region plus the input routing bound to it), and the **root** that hosts them. "User," "account," "login," and "corporate single-sign-on" are all policy inside a swappable login program. The OS exposes two primitives, unseal a realm given a credential-operation and bind a seat to a Matrix, and everything else is the login program's business. That is why a login program can be wired to a local password, a corporate VPN, a cloud identity provider, or a biometric with no change to the mechanism: it boots a Matrix and the OS hands it the seat.

## The decided mechanism

### Realm unlocking is a general primitive, not a login feature

`unseal(realm, credential-operation) → key released into the realm's own domain` is one OS primitive with several callers:

- the **boot login program**, which unseals the top-level realm and then rebinds the seat to the Matrix;
- any **app or shell hitting a sealed sub-realm**, such as the password-protected vault inside your files, which unseals on demand and re-seals when done with no seat rebind, so you stay where you are;
- a **nested login**, which unseals a child Matrix.

Login is unseal plus seat-rebind. An in-place vault unlock is unseal only. Login is not special; it is the top-level instance of a general mechanism, which is why it nests: a custom login program can log into a nested Matrix, recursively.

### The login program is a confined sibling-broker, never the Matrix's host

Login runs as a **transient, confined broker** holding only a `Capability<VerifyCredential>` and, during authentication, the seat to render its prompt. It is not sandbox-free and it is not root. The flow:

1. Root lends the seat to the broker to run authentication.
2. The broker authenticates by whatever policy it implements (local password, corporate VPN, cloud provider). Its output is an authorization to unseal a specific realm. It never holds the realm key; it has only the credential-operation.
3. The broker hands that authorization back to root. Root performs the unseal, releasing the key into the new Matrix's domain, and spawns the Matrix as a direct child of root.
4. Root reclaims the seat and binds it to the Matrix. The broker exits.

The Matrix is a sibling of the broker, both children of root. The broker does not host or run the Matrix. Hosting it would interpose the login program permanently in the session's trusted base, seeing all input and output forever. Root is already the irreducible root of trust, so the Matrix is a child of root, not of the broker. "The broker holds no user authority" is then structural rather than a discipline. It is provable from the broker's reach ceiling: it never holds the realm root, and the mint delivers into the seat and Matrix, not through the broker.

This confinement is what makes a swappable or corporate login program safe. It can only fail to authenticate, never exfiltrate the realm. Two anti-phishing details complete it:

- **Local credential entry is root's secure-entry modality** ([[windowing_and_compositor]]). The human types into a root-owned field, so even a malicious broker sees only "authenticated / not," never the raw password.
- **Remote authentication** runs in the broker's contained webview to the identity provider. The credential goes to the remote service, and the broker is held to browser-equivalent risk, no worse.

### The session is a seat-bound lease

A session is a leased delegate of the identity-world principal, scoped to a seat for a bounded window. It is principal-shaped, since it bears authority, but it is not a new identity kind. Its bundle at mint is minimal: the realm root (released by the credential), the seat routing, and the user's identity. It is not "everything the user can do." Apps started in the session get fine-grained capabilities minted down by gesture ([[human_permission_ux]]), so least authority is the default path. The derivation rule: a **personal** session holds the full realm root; a **work** session holds the realm root intersected with the organization's scoped ceiling ([[multi_user_and_org_control]]).

### Sealing, and the fallback without sealing hardware

The realm is strongly sealed by default through envelope encryption: one random data key, unlockable through any of N **wrapped** credentials (passphrase, biometric, a disclosed user-held escrow, a backup key), sealed to the credential plus the measured-good boot ([[secrets_and_keys]], [[boot_and_trust_chain]]). A tampered boot leaves it sealed, and the credential never passes through the broker, because a secure element performs the match. "Any one of N" is plain key-wrapping.

"Forgot my password" recovers through a pre-provisioned method, never an OS backdoor. Recovery is a pre-provisioned, disclosed, user-or-host-held escrow credential: other devices, chosen guardians, or an opt-in cloud or org support path. A high-security realm can opt out of it. The mechanism is in [[secrets_and_keys]].

When sealing hardware is absent, the realm degrades visibly. The key is derived from a passphrase (KDF) and the realm stays encrypted, but the session is marked unattestable. The boot-binding is gone and the OS says so, never silently.

### Compositing is a seat-bind, not a separate right

There is no "compositing" handed over as a subsystem. The compositor is trusted root infrastructure. A Matrix gets a surface plus input routing, which is the seat bound to it. "Full-screen" means the whole region the parent grants (whole-parent-view). A parent owns a compositor region and grants sub-regions to its children, and the window-resize and region-grant protocol is a compositor implementation detail ([[windowing_and_compositor]]), identical for a windowed app or a full-screen Matrix.

Lock and timeout confirm the broker is never load-bearing. Root reclaims the seat from the Matrix, re-seals the realm by dropping the key, and spawns a fresh login broker. The Matrix is suspended and sealed, never dependent on the broker staying alive.

### The login chooser sees only a minimal, opt-in unsealed label

A login chooser wants to show "John Smith [avatar]" before unlock, but everything inside the realm is sealed until then. The sealed realm is the home of all user config (name, avatar, settings), configured from within and shown by the Matrix itself after unlock, so "Hello John" needs nothing from the broker. There is one exception: a minimal, unsealed label (name plus avatar, nothing more) that the Matrix exports from its settings to an unsealed sidecar, readable by the chooser while the realm is sealed. It is opt-in. Export the label for a friendly chooser, or export nothing for a bare prompt, the private lock screen. The bound "anything beyond a name and picture is too much" is enforced structurally: that label is the only thing outside the seal, so there is nowhere to leak more.

### The granting structure: Matrices forward, root mints

Login bottoms out at the same recursion every host does. A Matrix is a host that mediates each child capability request four ways: deny, synthesize, forward-through-a-membrane, or forward-whole. Synthesize hands the child a contained fake it cannot distinguish. Forward delegates real authority, attenuably and revocably. A Matrix holds (declares) only the authority it forwards for real; everything synthesized needs no declaration. A specialized Matrix shipping a known child (a portable-world closure) can therefore be compiled to declare exactly the forwarded-real subset, while a general Matrix that hosts arbitrary apps holds a broad runtime envelope and attenuates down per child. "Matrix" is a role a component plays, not a new primitive, and the default implementation is a config-driven forward/synthesize/deny mediator that custom ones override.

The recursion ends where authority stops being forwarded from above and starts being minted because the component owns the device. That hard stop is the trusted core as the origin of authority, distributed across its resource owners. The network stack mints flow capabilities, the compositor mints surfaces and seats, the scheduler mints CPU and memory, the storage manager mints realm roots and runs `unseal`, and each driver mints its device, all unified by the capability system. The **boot-root** ("root = the bare machine," [[identity_and_principals]]) holds the top of every origin and delegates the first slice to the login broker. Root is the one component with no manifest: its authority is hardware-derived, not requested from a host. Root mints, every Matrix forwards or synthesizes, and a request walks up the host chain until either a Matrix synthesizes it (the hard stop for synthetic authority) or it reaches the resource owner that mints it against the metal (the hard stop for real authority).

## Concerns & Design Space

- **The session as a leased principal.** A leased ([[capability_lifecycle]]) delegate carrying a human's authority for a bounded window. Everything the user's apps hold is minted down from it by gesture.
- **On-demand and nested sealing.** Any subtree can be its own sealed realm with its own credential, unsealed on demand and re-sealed when done, through the same `unseal` primitive without a seat rebind. The prompt runs on the trusted path, so the requesting app triggers the unlock but never sees the credential or the key ([[secrets_and_keys]], [[filesystem_as_database]]).
- **Continuous and passwordless authentication.** A session is a lease whose renewal is gated by a presence signal (proximity, the OS-attested physical seat), and loss of signal lets it expire (auto-lock). A high-stakes action demands **step-up**: a fresh strong-auth mint for a short-lived elevated capability ([[human_permission_ux]]). The seat-binding is the continuous-auth substrate.
- **Multi-user and switching.** Each user is a seated principal (identity-world plus seat). Switching routes the seat to a different identity-world, which is a separate login. Two sessions never share ambient authority; isolation is structural (no global root), not a tag check ([[multi_user_and_org_control]]).
- **Remote and federated login.** Authenticating against an organization or remote principal to the same standard as a local one. The OS brokers the credential, so the service receives a scoped authentication and never the user's password, and federated login can mint a per-relationship identity ([[wallet_and_credentials]], [[distributed_boundary]]).
- **Zero value.** A zeroed session is the unauthenticated pre-login principal ([[omega_substrate]]): a valid principal holding no capabilities with the realm still sealed, so not-yet-logged-in is the default and an uninitialized session reads nothing.

## Key Questions

- What is in the session bundle at mint? Only the realm root, the seat routing, and the identity, derived by releasing the realm key on credential and intersecting with any governing org ceiling. Richer authority is minted down by gesture, never held ambiently.
- What is the realm key sealed to, and what happens without sealing hardware? It is sealed to the credential (an operation-capability matched by a secure element) plus the boot measurement. Without the hardware, a passphrase-KDF keeps the realm encrypted but the session is marked unattestable.
- Is a session a new principal kind or a lease? It is a seat-bound lease over the identity-world principal, not a new identity kind.
- How is "the broker holds no user authority" guaranteed? Structurally, by the sibling-broker shape: the mint delivers into the seat and Matrix, the broker only triggers it, and credential capture is root's secure-entry.

## Omega Leverage

- A session is a principal ([[identity_and_principals]]) holding capabilities ([[capability_model]]) under a lease ([[capability_lifecycle]]), with no new machinery.
- Credentials as operation-capabilities (`Capability<VerifyCredential>`) reuse the secrets model ([[secrets_and_keys]]).
- Sealing binds to the boot measurement, the trust-chain artifact ([[boot_and_trust_chain]]).
- The broker's "no user authority" is a reach-ceiling fact, not a runtime promise.
- Omega does not define credential or attestation protocols; those are runtime and provider structure Cathedral specifies.

## Open Questions

- Which recovery cryptography goes beyond plain key-wrapping? "Any one of N" wrapped unlock keys is the base. A K-of-M threshold-sharing scheme and first-pin (guardian-key trust) are open and belong with the other security-review questions.
- How are organization-mandated re-authentication intervals and revocation propagated to a live session lease across a partition ([[distributed_boundary]])?

## Related
- [[identity_and_principals]] — a session is a leased delegate of the identity-world; root = the bare machine.
- [[secrets_and_keys]] — credentials as operations and realm-key sealing.
- [[capability_lifecycle]] — the session as a lease; continuous auth as renewal.
- [[multi_user_and_org_control]] — users as seated principals; the org ceiling-intersection.
- [[filesystem_as_database]] — the user realm the session unseals.
- [[boot_and_trust_chain]] — sealing to a measured boot; visible degradation without sealing hardware.
- [[windowing_and_compositor]] — the seat, the region grant, and secure-entry.
- [[human_permission_ux]] — minting down by gesture; step-up authentication.
