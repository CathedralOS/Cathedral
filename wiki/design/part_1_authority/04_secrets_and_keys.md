# Chapter 04: Secrets & Key Management

> Every OS eventually grows a keychain; this one is designed in from the start. The chapter owns secrets, and the idea that a secret is an operation, not bytes.

## The Legacy Model

Legacy systems leak secrets by construction. Credentials live in dotfiles, config files, and environment variables. Private keys sit on disk readable by whatever the uid can read. An ambient keychain or agent hands raw key material to any process that asks nicely. The unit of sharing is the secret itself: you give a component the bytes of the API token or the private key, and from that moment the system has lost control of it. Rotation means re-issuing and chasing down copies. There is no per-component scoping, no leasing, and no notion that what a component needs is the ability to sign, not the key.

## The Cathedral Model

A secret is an **operation capability**, not a payload. Components do not receive key material. They receive narrowly-scoped authority to use a key for a specific operation, brokered by a holder that keeps the bytes, ideally in hardware that never exports them.

```omega
Capability<SignWithKey(K, alg: Ed25519)>
Capability<DecryptWithKey(K)>
Capability<UseToken(T, scope: "read:contacts")>
```

A component that needs to authenticate gets `UseToken(T, scope)`, not the token. A component that needs to sign gets `SignWithKey(K)`, not the private key. Leasing, attenuation, and revocation then work the same way they do for every other capability ([[capability_lifecycle]]). You can hand out a signing operation that expires, that is scoped to one algorithm, and that is revoked without rotating the underlying key. The hardware root of the key chain is anchored at boot ([[boot_and_trust_chain]]).

### Realm sealing: envelope encryption with N wrapped keys

A sealed realm is encrypted once under a single random data key, and each way to unlock it is a separately stored copy of that data key wrapped under a key-encryption-key derived from one method. The methods are a passphrase, through a slow KDF that is hardware-entangled and rate-limited so a weak PIN resists offline attack; a biometric, with the enclave releasing a key it already holds; an escrow credential; or a written backup key. N unlock paths are N wrapped copies of one data key. The data is never re-encrypted. Adding a method wraps another copy. Revoking one deletes its copy, and the data key is rotated if it may have been cached. "Any one of N" is plain key-wrapping and is shipping-grade. "K-of-M required" is secret sharing (threshold crypto) and stays with first-pin in the security bucket.

### Recovery is pre-provisioned escrow, not a backdoor

Strong sealing is on by default, so a stolen device reveals nothing. We do not use the weak-gatekeeper model. Recovery is pre-provisioned unlock methods, not a backdoor. The default credential set includes a disclosed, user-or-host-held escrow: your other devices, chosen guardians, or, opt-in, a cloud or organization credential whose support path can recover you. The high-security realm opts out of escrow. Lose the factor and the data is gone, as in iOS Advanced Data Protection. Cathedral's one constraint over the normal default is that escrow is disclosed and user-or-host-held, never a covert vendor key.

## Concerns & Design Space

- **Operation-not-bytes as the default.** Raw-secret access is the exceptional, audited case; the normal interface is a scoped operation capability. Avoid handing components material whenever a brokered operation will do.
- **Hardware-backed keys.** Keys live in a secure element or TPM and never leave it. The OS holds an operation handle, not the key, and the boundary provider for the crypto operation is the hardware itself ([[driver_model]]).
- **Per-app / per-component scoping.** Each principal ([[identity_and_principals]]) gets its own derived, narrowly-scoped operations. There is no shared ambient keychain.
- **Delegated decryption.** A component can be granted the ability to decrypt a specific blob without ever holding the decryption key. This is delegation of an operation, recorded as an edge in the graph.
- **Human / biometric unlock.** Unlock is a human-driven capability-minting event. A passphrase or biometric releases a leased operation capability, not the key. Biometrics derive no key from the fingerprint, which is fuzzy and not secret. The secure element matches the template internally and then releases a key it already holds. The fingerprint is an authorization gesture to the hardware, a presence-gated convenience over the passcode, which stays the cryptographic root and is required after reboot.
- **Realm sealing.** N unlock paths are N wrapped copies of one data key (see "Realm sealing" above).
- **Recovery & rotation.** Recovery is disclosed, user-or-host-held escrow (see "Recovery is pre-provisioned escrow" above). Rotation rewrites which key backs an operation without invalidating the operation's holders where policy allows.
- **Leasing.** Network credentials, API tokens, and session secrets are leased by default ([[capability_lifecycle]]). Expiry is the primary revocation path.
- **The credential zoo.** Network credentials, API tokens, certificates, passkeys, and org-managed keys are all the same shape: a scoped operation over a secret held elsewhere.
- **Payments: the Warden as a secure element.** A payment credential is operation-not-bytes taken to its sharpest. The Warden holds tokenized device credentials and keys, never raw card numbers (PANs), following the Apple Pay model. There is no reusable bearer secret to steal, and the system stays out of the PCI-DSS scope a card vault would drag in. The operation is signing a per-transaction cryptogram over `(amount, payee, nonce)`, which is un-replayable and un-rerouteable, gated by a local biometric unlock that never leaves the device. The Warden lives outside the user Matrix, at root, since holding it inside is needless risk, and it is grant-chain-forwardable into an app like any capability.
- **Authorizing a charge.** The human gesture that authorizes a charge is a WYSIWYS confirm minting a one-shot `Charge` capability, which is [[human_permission_ux]]'s action-confirm. The auth, capture, and settle chain it drives is output-commit ([[transactions_and_consistency]]). Reversibility (chargebacks) is a separate adjudication service, not a property of the primitive.
- **Org-managed keys.** An organization ([[multi_user_and_org_control]]) may own keys and delegate bounded operations to enrolled devices and users.
- **Zero value.** A zeroed signing or decryption operation is the fail-safe sentinel (shape 4 in [[omega_substrate]]). Invoking it yields a clearly-invalid result that verification rejects, never a forged signature or a silent plaintext, so an uninitialized key capability fails visibly rather than dangerously succeeding.

## Key Questions

- What is the canonical form of an operation capability over a secret, and how does it bind to the hardware-held key without exposing it?
- When is raw-secret access ever legitimate, and how is that exception audited?
- How does rotation rewrite the backing key for an operation without a revocation window for legitimate holders?
- What threshold-sharing scheme backs a K-of-M realm unlock, and how is guardian-key trust established (first-pin)?

## Omega Leverage

- A secret operation is a capability value plus domain (`Key::Signing`, `Token::Scoped`), the same machinery as every other capability, with no new keyword.
- Leasing, attenuation, and revocation come straight from [[capability_lifecycle]]. A leased `SignWithKey` is a capability with an expiry.
- The crypto operation is a `boundary` with a hardware provider. Its Warden/crypto service identity appears in the reach row, and ordinary code reaches it only through the held capability.
- Selected wire codecs over ordinary numbered schemas carry scoped tokens and certificates across boundaries.
- Omega has no native "operation handle over an at-rest, hardware-held secret that survives reboot without becoming forgeable". This is the same serialized-capability gap flagged in [[capability_lifecycle]], sharpened for keys.

## Open Questions

- Can the type system guarantee a component never obtains raw bytes when it was granted only an operation, even across IPC and serialization?
- How are operation capabilities revoked when the holder is offline or partitioned ([[distributed_boundary]])?
- Where does the very first key come from at boot, and who attests it ([[boot_and_trust_chain]])?

## Related
- [[capability_model]] — secrets are capabilities like any other.
- [[capability_lifecycle]] — leasing, rotation, and revocation of operations.
- [[identity_and_principals]] — the principals keys make unforgeable.
- [[boot_and_trust_chain]] — the hardware root of the key chain.
