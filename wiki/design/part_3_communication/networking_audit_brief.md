# Networking — Audit Brief & Crunch List (WORKING, NOT CONVERGED)

> Status: this is the output of a multi-agent audit/synthesis, kept as a working
> artifact to deep-dive from. **Nothing here is a decided design** — the settled
> spine is small; most of this is a map of what still needs crunching. Do not
> treat as a chapter. Bank into `00_ipc_and_service_invocation.md` /
> `02_distributed_boundary.md` / a future networking chapter only after a topic
> converges in conversation.

## The settled spine (mostly already implied by existing chapters)
- **Flow-as-capability.** A network capability is a *flow authorization* — the
  right to exchange packets with a named, identity-pinned peer, enforced at the
  shared NIC. Not "can network," not a socket. Default-deny = absence of a grant.
- **Ports deleted locally, demoted on the wire.** No listening integer inside the
  machine; a service *is* its endpoint capability ("bind" = register an endpoint).
  Kills `EADDRINUSE`, privileged `<1024`, `SO_REUSEPORT` theft, ephemeral
  exhaustion. A port's three fused jobs split: **demux** → broker steering;
  **identity** → pinned peer key; **authority** → held flow capability.
  FORCED: the wire still carries a 16-bit port; a public server must listen on
  443/53/22 and demux *after accept*.
- **Send vs recv are two capabilities; the session is bidirectional.**
  Egress = `Reach<Peer>`; ingress = `Serve` (reachable-as-key). Independently
  grantable/revocable; once established a flow is bidirectional like local IPC.
- **Per-Matrix `127.0.0.1`.** Loopback resolves inside the Matrix's own
  resolution environment (the Matrix asks for its *own* loopback grant; no
  rethrow to the outer Matrix). Localhost-daemon disaster class uninstantiable
  for native code.
- **Demux's four-line contract — does exactly:** (1) admission (drop
  non-authorized peers at flow-install); (2) **cross-delivery prevention** (the
  one provable invariant — lands in the right world's inbox); (3) flow-table
  arbitration (finite HW steering slots; overflow → software); (4) egress
  pacing / fair-queue. **MUST NOT:** parse/terminate transport, hold redeemable
  standing authority (route handed *tickets*), see plaintext, trust the NIC
  driver for routing. Route on an endpoint-supplied **routing token**
  (flow-handle / QUIC Connection ID / 5-tuple), never on semantic state.
- **Transport lives in each world** (TCP/QUIC/TLS/congestion/NAT-traversal as a
  userspace library; exokernel/Demikernel direction). Congestion fairness is the
  scheduler's job: per-world paced `EgressBudget`, fair-queued.

## The genuinely-hard residuals (the crunch list)
1. **First-pin / cold-stranger trust** (THE keystone) — crypto is solved, the
   human cold-trust bootstrap is not; dominant attack is non-cryptographic
   (transparency-logged confusable lookalike). Needs: confusable/mixed-script
   hard-blocks vs the user's own pins, refuse-secret-surface-from-low-pin,
   born-low provenance-gated ceilings, k-of-n OS-vendor root, store-introducer
   quorum. (Already parked as needing security expertise — see
   `../speculation/network_trust_fabric.md`.)
2. **Relay economics + commons** — inbound behind CGNAT *requires* a relay;
   relay demand concentrates on the exact battery/CGNAT devices the commons asks
   not to serve, so the vendor-seeded relay floor is a permanent recurring cost +
   a centralization/censorship chokepoint, not a cold-start crutch. "No central
   servers" holds for *integrity*, not *availability* behind CGNAT.
3. **Legacy public-ingress front-end** — a stock browser does DNS→SYN to :443
   with no key, so the broker MUST accept unauthenticated TCP/TLS on a fixed
   port, parse ClientHello SNI, and eat SYN-flood DoS like today. A carved-out
   attack-exposed trusted component, an explicit exception to "never parse."
4. **Demux token discipline under QUIC** — broker-minted/partitioned Connection-ID
   space (QUIC-LB style); the cross-delivery invariant only provably scopes to
   broker-minted tokens, degrading to 5-tuple for stock peers.
5. **The open-web client tier** — browser/`curl`/paste-a-URL reach peers that
   don't exist at build time → a loud gesture-gated `Reach<UserDirected>` tier
   where the URL bar/picker *is* the per-connection mint (network analogue of the
   file picker). Plus the POSIX porting shim (bind→endpoint-register,
   connect→authorize) and a zero-ceremony developer-Matrix for the inner loop.
6. **"Locally-terminated / synthetic" fact vs Matrix indistinguishability** —
   a positive "this flow never left the machine" indicator is meaningful only to
   a child that brought its own pinned anchor; for confined code, "are you really
   online" must stay unanswerable by construction. Don't promise an honest-offline
   indicator.
7. **Finite steering-slot allocation** as a `SteeringSlot` capability + its
   cross-world timing side-channel (slot occupancy leaks).
8. **Recovery-without-a-backdoor** + per-relationship-key cross-device continuity
   (unlinkable-by-default, linkable-by-explicit-intent via the user's own
   authority graph).

## Honest ceilings (cannot be engineered away)
- **Tunnel-over-authorized-flow exfil is NOT closeable** — capabilities bound the
  *peer set*, not the *bytes*. Permanent ceiling of the whole capability model;
  only conjunction ceilings (`deny Read<Photo> ∧ network_io`) structurally help.
- **Metadata always leaks** — pinning hides content/identity, never
  existence-of-flow, timing, or the social graph.
- **Per-peer egress is unenforceable at the legacy CDN edge** — one IP fronts
  ~50k tenants, ECH hides SNI; the grant authorizes the whole shared front.
- **Inbound reachability is not the local OS's to solve** — the public mapping
  lives in upstream middleboxes (CGNAT/ISP); the OS can run ICE + prefer IPv6 +
  relay, but cannot conjure a public address.
- **The headline native wins (mobility, no-port-config, no phoning-home) scale
  with the Cathedral-native + store-mediated fraction of a user's peers — near
  zero at launch.** Day-one value is vs the *legacy* internet: no ambient exfil,
  per-app egress attenuation, one observable authority graph, QUIC mobility.

## p2p / future, grounded (commit-now vs speculative)
- **Commit now:** bind a flow to a self-certifying peer key (onion-v3 / libp2p
  PeerID / npub); carry identity as a QUIC Connection ID (mobility default);
  WireGuard's `peer-key→allowed-IPs` is the coarse prior art flow-as-capability
  generalizes. A Cathedral network provider *is* Tailscale/Nebula (resolve, authorize, relay).
- **Urbit:** take key-as-identity + scry read-only namespace; reject scarce
  saleable address space, the bespoke VM, the sponsorship hierarchy.
- **Speculative (keep parked):** UDP/QUIC isn't universally passable (TCP
  fallback re-inherits the 4-tuple); `name→key` for a stranger still needs
  consensus; `address=pubkey` routing (Yggdrasil/cjdns) doesn't scale to internet
  size.

## Capability-purity flags to resolve
- **Contribution commons** "default-on by being a device" is role-blessed
  authority → make it a held, visible, revocable `Commons::Serve(budget)`.
- **Rendezvous DHT** is the one flat global namespace Cathedral otherwise avoids
  → quarantine as *location-hint only*, never identity/authority.
- **Root resolver** must be a held `Resolve<namespace>` capability, not an
  ambient "resolve any name" service.
