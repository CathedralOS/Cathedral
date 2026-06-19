# Speculative: Post-DNS Resolution — self-certifying keys + an untrusted commons

> **Status: SPECULATIVE — a forward-looking exploration, not committed design.** Captured 2026-06-19. What a post-DNS naming/resolution substrate could look like. Companion to [network_trust_fabric.md](network_trust_fabric.md): the trust fabric handles *auth / secrets*; this handles *naming / locating*. The URL bar of the [future browser](future_browser.md) is the one real consumer of its hard part.

## The core move: split identity from location (from trust)

DNS fuses three jobs — **identity** (who is `bank.com`), **location** (which IP), and (via the CA) **trust** (this cert *is* that name). Unbundle all three:
- **Identity = a self-certifying key.** The identifier *is* (or derives from) a public key, so it cannot lie about who it is. Proof of ownership = sign a challenge nonce, verify with the key. Trivial.
- **Location = an untrusted lookup.** `key → current endpoints`, resolved through infrastructure that needs **no trust**, because the records are signed by the key (a wrong answer just fails the connection handshake).
- **DNS-as-addressing may survive as a dumb hint** (one endpoint among several); **DNS-as-identity and the whole CA layer die.**

Not hypothetical — existence proofs ship today: Tor v3 `.onion` addresses *are* Ed25519 public keys; libp2p peer IDs are key hashes; Nostr `npub` is a pubkey; W3C **DIDs** standardize "self-certifying id + pluggable resolver."

## Two lookups — only one is hard

- **key → location** — an **untrusted DHT** (Kademlia-style). No consensus, no authority: nodes store signed location records; a lying node can misdirect (harmless — the wrong box can't authenticate as the key) but cannot forge. **This is most of the system**: apps ship with the key, they just need to find *where* it is now.
- **name → key** — binding a *scarce, human-meaningful* name (`amazon`) to a key needs **consensus** (squatting, transfer, global agreement). This is the genuinely hard layer — hard for *governance* reasons, not crypto ones — and it is needed **only** for cold discovery of a stranger by typed name (the URL-bar minority). Apps, links, bookmarks, and established relationships are all key-native and skip it.

So the CA/DNS-shaped problem shrinks to *one operation*. For it, the binding is a **pluggable, opt-in fork**: a CA-as-bootstrap-notary (TOFU over existing TLS, used once then pinned), an **ENS/Namecoin/Handshake**-style ledger, or **GNS/petnames** (drop global names, use introduction). A values/business choice, not a spine dependency.

## How phone-home works

```
app holds:  { backend_key: 0xAB…, seed_hints:[dht_bootstrap, relay_R3] }   // shipped by the store
  1. records = DHT.lookup(backend_key)              // untrusted directory, keyed by the exact key
  2. keep only records whose signature verifies against backend_key   // O(1) discard of spam
  3. connect(endpoint); challenge-response vs backend_key             // wrong box can't complete it
```

The resolver is **untrusted**; mobility is free (republish records on move); censorship-resistance comes for free (no central registry). Spam is a non-issue for *integrity* — you query an *exact key*, discard invalid-signed records, and different keys live in different DHT buckets. The only real attacks are **availability**: flooding (mitigated by publish-cost / TTL / per-key sharding) and **eclipse** (surround a key's keyspace region to *hide* the real record — censorship, **not** impersonation; mitigated by K-way replication, independent bootstrap nodes, and the seed hints the app shipped with).

## Who hosts it

Nobody and everybody — the **participants**. Records live on the K nodes closest to each key; membership is the distributed union of everyone's routing tables, kept fresh by liveness pings + peer-exchange gossip. Because records are signed, **a hosting node has no authority** — so "who hosts it" is an *availability* question, not a *trust* one.

The one irreducible seed: a brand-new node needs one pre-known entry point — a hardcoded, **plural, replaceable bootstrap list**. This is the **network-layer twin of first-pin**: membership, like trust, must be imported from one out-of-band seed; it can't be conjured from nothing. It fails the same safe way — a malicious bootstrap can *eclipse/censor* a joining node but cannot *impersonate*.

## The availability answer only an OS can give: a contribution commons

Availability isn't free — *someone* must keep enough honest nodes online. An OS (unlike an app) can make participation a **default substrate behavior**: every capable Cathedral device serves the routing/resolution commons by default, so the network self-hosts and scales with the install base — **no trusted central servers, no payment rails**. Refinements:
- **Capability-weighted, not flat.** Always-on / mains-powered / unmetered nodes carry storage + routing; metered/battery devices are query-mostly. Net-positive *in aggregate*, scaled to each device's means (libp2p's DHT-server vs DHT-client split, made an OS policy).
- **A budgeted, capped, governed resource** ([[scheduler_and_resources]]) — a bounded slice that can never starve the user's own work.
- **Default-on beats precise ratio-metering.** BitTorrent built the largest DHT on earth purely by making participation the client default; a literal per-device "give 110% of what you use" accounting is over-engineering. The *intuition* — the capable default must be net-positive — is the real lever.
- **Vendor-seeded baseline as an availability floor.** Well-provisioned bootstrap/storage nodes for cold-start and thin-coverage regions — **no integrity authority** (signed records), just keeping the lights on until the install base is dense.

## Apps need no naming authority at all

The app store is the **introducer.** At install it provisions the app with its backend's **key** (self-certifying); the app authenticates its backend by pinned key and locates it via the DHT — no CA, no DNS-trust. The human-meaningful part is just the app's name *in the store's curated namespace*. **One pin** (the store, at OS install, protected by measured boot) bootstraps the entire app ecosystem.

## Reliability

DNS is a recurring single-ish point of major internet outages. A K-way-replicated overlay with **no central resolver** is architecturally *more* available for the resolution function — nothing central to drop, graceful degradation. Honest caveat: a *young* overlay has its own teething (churn, eclipse, cold-start), so "beats DNS" is a **maturity** claim, not an automatic one.

## Value capture is an open, per-layer business dial — NOT decided

"Anti-authoritarian" is **not a settled premise**; the economics differ by layer:
- **Resolution / DHT = a cost center.** The commons is *cost-shedding* — fiscally smart, not charity. (No one ever got rich selling name resolution.)
- **Store, identity/attestation/introducer, payment rails = moats.** Capturable; giving them away would be foolish. Killing tracking-cookies doesn't forfeit value — it picks the premium/paid model over ad-surveillance (Apple's playbook).
- The governing variable is **adoption-elasticity + regulatory exposure per layer**, not ideology. A cold-start OS has *no pricing power* — it must be *more* open early to win the network effect, then ratchet capture once entrenched.
- **The one technical constraint:** capturing the trust *root itself* recreates the single-point supply-chain risk **and** breaks the product ("no central authority can betray you"). Monetize **services on top** of a credibly-neutral root, not the root.

So the **mechanism is deliberately capture-agnostic**: the store-as-introducer supports both a mandatory-cut store *and* permissionless pinning; identity supports both paid attestation *and* free key-pinning. Build the dial; defer the policy.

## Prior art
libp2p / Kademlia (+ **IPNS** signed mutable pointers); BitTorrent Mainline DHT (scale); Tor onion v3 / HSDir (self-certifying name + DHT + censorship-resistance, with blinded-key privacy); ENS / Namecoin / Handshake (blockchain name→key); GNUnet **GNS** (petname/delegation, no chain); W3C **DIDs** (self-certifying id + pluggable resolver); **Nostr** (key-as-id, relay discovery); Hyperswarm / Hypercore. As with the trust fabric, the novelty is **composition + making participation an OS default**, not new crypto.

## Related
- [network_trust_fabric.md](network_trust_fabric.md) — the auth/secret layer this sits under.
- [future_browser.md](future_browser.md) — the URL bar is the one consumer of name→key discovery.
- Cathedral [[naming_and_discovery]], [[networking]], [[distributed_boundary]], [[scheduler_and_resources]], [[identity_and_principals]].
