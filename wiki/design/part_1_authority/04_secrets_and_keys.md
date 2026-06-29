# Chapter 04: Secrets & Key Management

> Every OS eventually grows a keychain; this one is designed in from the start. The chapter owns secrets — and the idea that a secret is an *operation*, not bytes.

## The Legacy Model

Legacy systems leak secrets by construction. Credentials live in dotfiles, config files, and environment variables; private keys sit on disk readable by whatever the uid can read; an ambient keychain or agent hands raw key material to any process that asks nicely. The unit of sharing is *the secret itself* — you give a component the bytes of the API token or the private key, and from that moment the system has lost control of it. Rotation means re-issuing and chasing down copies. There is no per-component scoping, no leasing, no notion that what a component actually needs is the *ability to sign*, not the *key*.

## The Cathedral Model

The central inversion: **a secret is an operation capability, not a payload.** Components do not receive key material; they receive narrowly-scoped authority to *use* a key for a specific operation, brokered by a holder that keeps the bytes — ideally in hardware that never exports them.

```omega
Capability<SignWithKey(K, alg: Ed25519)>
Capability<DecryptWithKey(K)>
Capability<UseToken(T, scope: "read:contacts")>
```

A component that needs to authenticate gets `UseToken(T, scope)`, not the token. A component that needs to sign gets `SignWithKey(K)`, not the private key. This makes leasing, attenuation, and revocation work the same way they do for every other capability ([[capability_lifecycle]]): you can hand out a *signing operation* that expires, that is scoped to one algorithm, that is revoked without rotating the underlying key. The hardware root of the key chain is anchored at boot ([[boot_and_trust_chain]]).

## Concerns & Design Space

- **Operation-not-bytes as the default.** Raw-secret access is the exceptional, audited case; the normal interface is a scoped operation capability. Avoid handing components material whenever a brokered operation will do.
- **Hardware-backed keys.** Keys that live in a secure element / TPM and never leave it; the OS holds an *operation handle*, not the key, and the boundary provider for the crypto operation is the hardware itself ([[driver_model]]).
- **Per-app / per-component scoping.** Each principal ([[identity_and_principals]]) gets its own derived, narrowly-scoped operations; no shared ambient keychain.
- **Delegated decryption.** A component can be granted the ability to decrypt a specific blob without ever holding the decryption key — delegation of an operation, recorded as an edge in the graph.
- **Human / biometric unlock.** Unlock is a human-driven capability-minting event: a passphrase or biometric releases a leased operation capability, not the key. Biometrics derive *no* key from the fingerprint (it is fuzzy and not secret) — the secure element matches the template internally and then **releases a key it already holds**; the fingerprint is an authorization gesture to the hardware, a presence-gated convenience over the passcode, which stays the cryptographic root (required after reboot).
- **Realm sealing — envelope encryption with N wrapped keys.** A sealed realm is encrypted *once* under a single random data key; each way to unlock it is a separately-stored copy of that data key **wrapped** under a key-encryption-key derived from one method — a passphrase (through a slow KDF, hardware-entangled and rate-limited so a weak PIN resists offline attack), a biometric (the enclave releasing a held key), an escrow credential, or a written backup key. So **N unlock paths = N wrapped copies of one data key**: the data is never re-encrypted, adding a method wraps another copy, revoking one deletes its copy (rotate the data key if it may have been cached). "Any one of N" is plain key-wrapping (shipping-grade, decided); "K-of-M required" is secret *sharing* (threshold crypto) and stays parked with first-pin in the security bucket.
- **Recovery & rotation.** Strong sealing is **on by default** — a stolen device reveals nothing, never the weak-gatekeeper model — and recovery is *not* a backdoor but **pre-provisioned unlock methods**: the default credential set includes a **disclosed, user-or-host-held escrow** (your other devices, chosen guardians, or — opt-in — a cloud/organization credential whose support path can recover you), and the high-security realm opts *out* of escrow (lose the factor, the data is gone, as in iOS Advanced Data Protection). Cathedral's one constraint over the normal default: escrow is disclosed and user-or-host-held, never a covert vendor key. Rotation rewrites which key backs an operation without invalidating the operation's holders where policy allows.
- **Leasing.** Network credentials, API tokens, and session secrets are leased by default ([[capability_lifecycle]]); expiry is the primary revocation path.
- **The credential zoo.** Network credentials, API tokens, certificates, passkeys, and org-managed keys are all the same shape — a scoped operation over a secret held elsewhere.
- **Payments — the Warden as a secure element.** A payment credential is operation-not-bytes taken to its sharpest: the **Warden holds tokenized device credentials and keys, never raw card numbers (PANs)**, following the Apple-Pay model — so there is no reusable bearer secret to steal and the system stays *out of PCI-DSS scope* a card vault would drag in. The "operation" is signing a **per-transaction cryptogram over `(amount, payee, nonce)`** (un-replayable, un-rerouteable), gated by a local biometric unlock that never leaves the device. The Warden lives *outside* the user Matrix (at root — holding it inside is needless risk) and is grant-chain-forwardable into an app like any capability. The human *gesture* that authorizes a charge — WYSIWYS confirm minting a one-shot `Charge` capability — is [[human_permission_ux]]'s action-confirm; the auth→capture→settle chain it drives is output-commit ([[transactions_and_consistency]]). **Reversibility (chargebacks) is a separate adjudication service, not a property of the primitive.**
- **Org-managed keys.** An organization ([[multi_user_and_org_control]]) may own keys and delegate bounded operations to enrolled devices and users.
- **Zero value.** A zeroed signing or decryption operation is the fail-safe sentinel (shape 4 in [[omega_substrate]]): invoking it yields a clearly-invalid result that verification rejects and never a forged signature or a silent plaintext, so an uninitialized key capability fails visibly rather than dangerously succeeding.

## Key Questions

- What is the canonical form of an operation capability over a secret, and how does it bind to the hardware-held key without exposing it?
- When is raw-secret access ever legitimate, and how is that exception audited?
- How does rotation rewrite the backing key for an operation without a revocation window for legitimate holders?
- **Recovery without a backdoor — resolved (shape; crypto parked):** the realm is always strongly sealed (envelope encryption, N wrapped keys), and recovery is a **pre-provisioned, disclosed, user-or-host-held escrow** credential — opt-out for the high-security realm — never a backdoor. "Any one of N" is plain key-wrapping; the **K-of-M threshold-sharing scheme + first-pin** (guardian-key trust) stay in the security bucket.

## Omega Leverage

- A secret operation is a **capability value + domain** (`Key::Signing`, `Token::Scoped`) — same machinery as every other capability, no new keyword.
- **Leasing, attenuation, and revocation** come straight from [[capability_lifecycle]]; a leased `SignWithKey` is just a capability with an expiry.
- The crypto operation is a **`boundary`** with a hardware **provider** and a `device_io` effect; ordinary code reaches it only through the held capability.
- **`wire data`** carries scoped tokens and certificates across boundaries with stable field numbers.
- Omega has no native "operation handle over an at-rest, hardware-held secret that survives reboot without becoming forgeable" — this is the same serialized- capability gap flagged in [[capability_lifecycle]], sharpened for keys.

## Open Questions

- Can the type system guarantee a component *never* obtains raw bytes when it was granted only an operation, even across IPC and serialization?
- How are operation capabilities revoked when the holder is offline or partitioned ([[distributed_boundary]])?
- Where does the very first key come from at boot, and who attests it ([[boot_and_trust_chain]])?

## Related
- [[capability_model]] — secrets are capabilities like any other.
- [[capability_lifecycle]] — leasing, rotation, and revocation of operations.
- [[identity_and_principals]] — the principals keys make unforgeable.
- [[boot_and_trust_chain]] — the hardware root of the key chain.
