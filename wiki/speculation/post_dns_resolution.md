# Speculative: Post-DNS Resolution

> **Status: SPECULATIVE.** A forward-looking exploration, not committed design. Captured 2026-06-19. What a post-DNS naming and resolution substrate could look like: self-certifying keys plus an untrusted commons. Companion to [network_trust_fabric.md](network_trust_fabric.md): the trust fabric handles auth and secrets, and this handles naming and locating. The URL bar of the [future browser](future_browser.md) is the one real consumer of its hard part.

## The core move: split identity from location (from trust)

DNS fuses three jobs: **identity** (who is `bank.com`), **location** (which IP), and, through the CA, **trust** (this cert is that name). The post-DNS substrate unbundles all three:
- **Identity is a self-certifying key.** The identifier is, or derives from, a public key, so it cannot lie about who it is. Proof of ownership: sign a challenge nonce, verify with the key. Trivial.
- **Location is an untrusted lookup.** `key → current endpoints`, resolved through infrastructure that needs no trust, because the records are signed by the key. A wrong answer just fails the connection handshake.
- **DNS-as-addressing may survive as a dumb hint**, one endpoint among several. DNS-as-identity and the whole CA layer die.

Existence proofs ship today: Tor v3 `.onion` addresses are Ed25519 public keys, libp2p peer IDs are key hashes, Nostr `npub` is a pubkey, and W3C DIDs standardize "self-certifying id plus pluggable resolver."

## Two lookups: only one is hard

- **key to location** is an untrusted DHT (Kademlia-style). No consensus, no authority: nodes store signed location records. A lying node can misdirect (harmless, since the wrong box cannot authenticate as the key) but cannot forge. This is most of the system: apps ship with the key and just need to find where it is now.
- **name to key** binds a scarce, human-meaningful name (`amazon`) to a key, and that needs consensus (squatting, transfer, global agreement). This is the hard layer, for governance reasons rather than crypto ones, and it is needed only for cold discovery of a stranger by typed name, the URL-bar minority. Apps, links, bookmarks, and established relationships are all key-native and skip it.

The CA/DNS-shaped problem therefore shrinks to one operation. For it, the binding is a pluggable, opt-in fork: a CA-as-bootstrap-notary (TOFU over existing TLS, used once then pinned), an ENS/Namecoin/Handshake-style ledger, or GNS/petnames (drop global names, use introduction). A values and business choice, not a spine dependency.

## How phone-home works

```
app holds:  { backend_key: 0xAB…, seed_hints:[dht_bootstrap, relay_R3] }   // shipped by the store
  1. records = DHT.lookup(backend_key)              // untrusted directory, keyed by the exact key
  2. keep only records whose signature verifies against backend_key   // O(1) discard of spam
  3. connect(endpoint); challenge-response vs backend_key             // wrong box can't complete it
```

The resolver is untrusted. Mobility is free (republish records on move), and censorship-resistance comes for free (no central registry). Spam is a non-issue for integrity: you query an exact key, discard invalid-signed records, and different keys live in different DHT buckets. The only real attacks are on availability. Flooding is mitigated by publish-cost, TTL, and per-key sharding. An **eclipse** surrounds a key's keyspace region to hide the real record, which is censorship, not impersonation, and is mitigated by K-way replication, independent bootstrap nodes, and the seed hints the app shipped with.

## Who hosts it

The participants host it: nobody and everybody. Records live on the K nodes closest to each key. Membership is the distributed union of everyone's routing tables, kept fresh by liveness pings and peer-exchange gossip. Because records are signed, a hosting node has no authority, so "who hosts it" is an availability question, not a trust one.

The one irreducible seed: a brand-new node needs one pre-known entry point, a hardcoded, plural, replaceable bootstrap list. This is the network-layer twin of first-pin: membership, like trust, must be imported from one out-of-band seed, not conjured from nothing. It fails the same safe way: a malicious bootstrap can eclipse or censor a joining node but cannot impersonate.

## The availability answer only an OS can give: a contribution commons

Availability is not free, since someone must keep enough honest nodes online. An OS, unlike an app, can make participation a default substrate behavior: every capable Cathedral device serves the routing and resolution commons by default, so the network self-hosts and scales with the install base, with no trusted central servers and no payment rails. Refinements:
- **Capability-weighted, not flat.** Always-on, mains-powered, unmetered nodes carry storage and routing. Metered and battery devices are query-mostly. Net-positive in aggregate, scaled to each device's means (libp2p's DHT-server vs DHT-client split, made an OS policy).
- **A budgeted, capped, governed resource** ([[scheduler_and_resources]]). A bounded slice that can never starve the user's own work.
- **Default-on beats precise ratio-metering.** BitTorrent built the largest DHT on earth purely by making participation the client default. A literal per-device "give 110% of what you use" accounting is over-engineering. The intuition that the capable default must be net-positive is the real lever.
- **Vendor-seeded baseline as an availability floor.** Well-provisioned bootstrap and storage nodes for cold-start and thin-coverage regions. They carry no integrity authority (records are signed) and just keep the lights on until the install base is dense.

## Apps need no naming authority at all

The app store is the **introducer**. At install it provisions the app with its backend's self-certifying key. The app authenticates its backend by pinned key and locates it via the DHT, with no CA and no DNS-trust. The human-meaningful part is just the app's name in the store's curated namespace. One pin (the store, at OS install, protected by measured boot) bootstraps the entire app ecosystem.

## Reliability

A K-way-replicated overlay with no central resolver is architecturally more available for resolution than DNS, a recurring single-ish point of major internet outages. Nothing central to drop, graceful degradation. The caveat: a young overlay has its own teething (churn, eclipse, cold-start), so "beats DNS" is a maturity claim, not an automatic one.

## Value capture: an open, per-layer business dial, not decided

"Anti-authoritarian" is not assumed as a premise. The economics differ by layer:
- **Resolution / DHT is a cost center.** The commons is cost-shedding: fiscally smart, not charity. No one ever got rich selling name resolution.
- **Store, identity/attestation/introducer, and payment rails are moats.** Capturable, and giving them away would be foolish. Killing tracking-cookies does not forfeit value. It picks the premium/paid model over ad-surveillance, Apple's playbook.
- **The governing variable is adoption-elasticity plus regulatory exposure per layer**, not ideology. A cold-start OS has no pricing power. It must be more open early to win the network effect, then ratchet capture once entrenched.
- **The one technical constraint.** Capturing the trust root itself recreates the single-point supply-chain risk and breaks the product ("no central authority can betray you"). Monetize services on top of a credibly-neutral root, not the root.

The mechanism is capture-agnostic, so the dial gets built and the policy deferred. The store-as-introducer supports both a mandatory-cut store and permissionless pinning, and identity supports both paid attestation and free key-pinning.

## Prior art
libp2p / Kademlia (plus IPNS signed mutable pointers); BitTorrent Mainline DHT (scale); Tor onion v3 / HSDir (self-certifying name + DHT + censorship-resistance, with blinded-key privacy); ENS / Namecoin / Handshake (blockchain name-to-key); GNUnet GNS (petname/delegation, no chain); W3C DIDs (self-certifying id + pluggable resolver); Nostr (key-as-id, relay discovery); Hyperswarm / Hypercore. As with the trust fabric, the novelty is composition plus making participation an OS default, not new crypto.

## Related
- [network_trust_fabric.md](network_trust_fabric.md) — the auth/secret layer this sits under.
- [future_browser.md](future_browser.md) — the URL bar is the one consumer of name-to-key discovery.
- Cathedral [[naming_and_discovery]], [[networking]], [[distributed_boundary]], [[scheduler_and_resources]], [[identity_and_principals]].
