# Chapter 07: Secrets & Key Management

> Every OS eventually grows a keychain; this one is designed in, not bolted on. The chapter owns secrets — and the idea that a secret is an *operation*, not bytes.

## The Legacy Contract

Legacy systems leak secrets by construction. Credentials live in dotfiles, config files, and environment variables; private keys sit on disk readable by whatever the uid can read; an ambient keychain or agent hands raw key material to any process that asks nicely. The unit of sharing is *the secret itself* — you give a component the bytes of the API token or the private key, and from that moment the system has lost control of it. Rotation means re-issuing and chasing down copies. There is no per-component scoping, no leasing, no notion that what a component actually needs is the *ability to sign*, not the *key*.

## What Cathedral Wants

The central inversion: **a secret is an operation capability, not a payload.** Components do not receive key material; they receive narrowly-scoped authority to *use* a key for a specific operation, brokered by a holder that keeps the bytes — ideally in hardware that never exports them.

```omega
Capability<SignWithKey(K, alg: Ed25519)>
Capability<DecryptWithKey(K)>
Capability<UseToken(T, scope: "read:contacts")>
```

A component that needs to authenticate gets `UseToken(T, scope)`, not the token. A component that needs to sign gets `SignWithKey(K)`, not the private key. This makes leasing, attenuation, and revocation work the same way they do for every other capability ([[04_capability_lifecycle]]): you can hand out a *signing operation* that expires, that is scoped to one algorithm, that is revoked without rotating the underlying key. The hardware root of the key chain is anchored at boot ([[25_boot_and_trust_chain]]).

## Concerns & Design Space

- **Operation-not-bytes as the default.** Raw-secret access is the exceptional, audited case; the normal interface is a scoped operation capability. Avoid handing components material whenever a brokered operation will do.
- **Hardware-backed keys.** Keys that live in a secure element / TPM and never leave it; the OS holds an *operation handle*, not the key, and the boundary provider for the crypto operation is the hardware itself ([[24_driver_model]]).
- **Per-app / per-component scoping.** Each principal ([[05_identity_and_principals]]) gets its own derived, narrowly-scoped operations; no shared ambient keychain.
- **Delegated decryption.** A component can be granted the ability to decrypt a specific blob without ever holding the decryption key — delegation of an operation, recorded as an edge in the graph.
- **Human / biometric unlock.** Unlock is a human-driven capability-minting event: a fingerprint or passphrase releases a leased operation capability, not the key.
- **Recovery & rotation.** Losing the unlock factor, or rotating a key, must not orphan the data it protected; rotation rewrites which key backs an operation without invalidating the operation's holders where policy allows.
- **Leasing.** Network credentials, API tokens, and session secrets are leased by default ([[04_capability_lifecycle]]); expiry is the primary revocation path.
- **The credential zoo.** Network credentials, API tokens, certificates, passkeys, and org-managed keys are all the same shape — a scoped operation over a secret held elsewhere.
- **Org-managed keys.** An organization ([[31_multi_user_and_org_control]]) may own keys and delegate bounded operations to enrolled devices and users.

## Key Questions

- What is the canonical form of an operation capability over a secret, and how does it bind to the hardware-held key without exposing it?
- When is raw-secret access ever legitimate, and how is that exception audited?
- How does rotation rewrite the backing key for an operation without a revocation window for legitimate holders?
- What is the recovery story when the unlock factor or the rooting hardware is lost, without that recovery becoming a backdoor?

## Omega Leverage

- A secret operation is a **capability value + domain** (`Key::Signing`, `Token::Scoped`) — same machinery as every other capability, no new keyword.
- **Leasing, attenuation, and revocation** come straight from [[04_capability_lifecycle]]; a leased `SignWithKey` is just a capability with an expiry.
- The crypto operation is a **`boundary`** with a hardware **provider** and a `device_io` effect; ordinary code reaches it only through the held capability.
- **`wire data`** carries scoped tokens and certificates across boundaries with stable field numbers.
- Omega has no native "operation handle over an at-rest, hardware-held secret that survives reboot without becoming forgeable" — this is the same serialized- capability gap flagged in [[04_capability_lifecycle]], sharpened for keys.

## Open Questions

- Can the type system guarantee a component *never* obtains raw bytes when it was granted only an operation, even across IPC and serialization?
- How are operation capabilities revoked when the holder is offline or partitioned ([[17_distributed_boundary]])?
- Where does the very first key come from at boot, and who attests it ([[25_boot_and_trust_chain]])?

## Related
- [[03_capability_model]] — secrets are capabilities like any other.
- [[04_capability_lifecycle]] — leasing, rotation, and revocation of operations.
- [[05_identity_and_principals]] — the principals keys make unforgeable.
- [[25_boot_and_trust_chain]] — the hardware root of the key chain.
