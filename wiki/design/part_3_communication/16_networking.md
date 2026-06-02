# Chapter 16: Networking

> Network access is a capability over an *intent* — "connect to this service
> speaking this protocol" — not raw permission to emit packets at an address.

## The Legacy Contract

In a traditional OS, networking bottoms out at the socket: a process opens a file
descriptor to an IP address and port and is then free to send and receive
arbitrary bytes. Everything that makes networking *safe* or *meaningful* — DNS,
TLS identity, service discovery, firewalling, per-app permissions, bandwidth
limits — lives in a scatter of separate, ambient subsystems that the socket layer
neither knows nor enforces. An app that can reach `socket(AF_INET, …)` can talk to
*anything* the routing table allows; "network permission" is binary and ambient.
The OS enforces *reachability*, never *intent*. It cannot answer "which services
may this app talk to, over which protocols, under whose identity, within what
budget?" because the socket abstraction discards exactly that information.

## What Cathedral Wants

Network access is a held **capability** plus a `network_io` **effect**, and the
unit of access is an *intent-typed connection to a named service speaking a known
protocol* — not a raw address:

```omega
let conn = Network::connect(
    service = "api.vendor.com",
    protocol = PaymentAPI::v3,         // a wire-data protocol, negotiated
    identity = principal.tls_identity, // see [[05_identity_and_principals]]
    cap = net,                         // Capability<Network::Reach<PaymentAPI>>
);
```

The OS broker resolves the name, enforces firewall/policy, pins TLS identity,
negotiates the protocol version, and accounts the bandwidth — *then* hands back a
typed protocol endpoint, identical in shape to a local IPC endpoint
([[15_ipc_and_service_invocation]]). The component never sees `34.117.12.9:443`;
it sees `PaymentAPI.v3`. Permission is granted to the *intent*, so revoking
"may reach PaymentAPI" is a precise graph operation, not a firewall rule edit.

## Concerns & Design Space

- **Naming, DNS & discovery.** Resolution is a brokered, auditable step; service
  discovery (local-first, mDNS-ish, registry-backed) yields *typed* endpoints,
  not bare addresses.
- **TLS identity.** Connections bind to principals on both ends
  ([[05_identity_and_principals]]); identity pinning and attestation are part of
  `connect`, not an afterthought.
- **Per-app, intent-scoped permission.** "May reach `PaymentAPI`" is a capability;
  "may emit arbitrary packets" should be unreachable for ordinary components.
- **Firewall as capability policy.** Egress/ingress rules become ceilings over the
  network capability graph rather than a parallel ambient ruleset.
- **Connection brokering.** A trusted broker mints connection capabilities,
  enabling pooling, multiplexing, and revocation without the app holding a raw fd.
- **Bandwidth & budgets.** Network is a metered resource; budgets are accounted
  via effects and governed by the scheduler ([[10_scheduler_and_resources]]).
- **Transport assumptions.** QUIC-ish streams, multipath, and connection
  migration as the baseline, so mobility and multihoming are normal, not heroic.
- **NAT traversal & local-first.** Peer connectivity and offline/degraded
  behavior are designed in; sync resumes when reachability returns
  ([[17_distributed_boundary]]).
- **Observability.** Every connection is a traceable, attributable object — which
  principal, which protocol, how much bandwidth, what identity — by construction
  ([[33_observability_and_introspection]]).
- **Protocol schemas on the wire.** The bytes are framed by `wire data`, so
  cross-version interop and compatibility reports apply to the network exactly as
  to local IPC.

## Key Questions

- What is the minimal trusted broker, and how does it stay small enough to be
  TCB-worthy while resolving names, pinning identity, and negotiating protocols?
- Can *any* component ever obtain raw packet access, and if so, under what
  attenuation and audit?
- How is intent reconciled with reality when DNS, routing, or the peer's protocol
  version disagrees with the requested capability?
- Where do bandwidth budgets live — per principal, per capability, per connection
  — and how do they compose?

## Omega Leverage

- **Capabilities as values** make "may reach `PaymentAPI`" a held, attenuable,
  revocable grant rather than an ambient socket right. See Omega
  [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **`effects` (`network_io`)** mark and account every crossing into the network,
  giving the boundary edge where the OS broker provides authority.
- **`wire data`** frames the protocol on the socket with stable field numbers and
  compatibility rules, so version negotiation is schema-driven. See Omega
  [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **`boundary` providers** are the natural home for the transport stack (TLS,
  QUIC, NAT traversal) below the proved Omega connection API.
- Omega does **not yet** model bandwidth as a metered effect or connection
  identity/attestation as a fact — extensions Cathedral drives.

## Open Questions

- Is "raw packet access" simply outside the model (handled only by privileged
  drivers, [[24_driver_model]]), or an attenuated escape hatch?
- How do intent-typed connections interoperate with the existing internet, where
  most peers speak neither Cathedral protocols nor capability identity?
- What is the offline contract — which capabilities remain usable, and how does
  the budget accounting behave under partition?

## Related
- [[03_capability_model]] — network reach as a capability in the graph.
- [[15_ipc_and_service_invocation]] — a connection is a protocol endpoint.
- [[17_distributed_boundary]] — networking as the path off the machine.
- [[05_identity_and_principals]] — TLS identity binds both ends of a connection.
- [[10_scheduler_and_resources]] — bandwidth as a governed budget.
