# Chapter 01: Networking

> Network access is a per-destination capability: the right to exchange packets with a named, identity-verified peer, enforced where packets meet the shared Network Interface Controler (NIC). The OS owns that demux and the authority; the transport (TCP, QUIC, TLS) is a userspace library.

## The Legacy Model

In a traditional OS, networking bottoms out at the socket: a process opens a file descriptor to an IP address and port and is then free to send and receive arbitrary bytes. Two things are wrong. First, the kernel owns one general-purpose TCP/IP stack that every app shares, which fixes the policy (its congestion control, its buffering, its feature set) and costs a syscall on every operation. Second, authority is ambient and binary: an app that can open a socket can reach anything the routing table allows. Everything that makes networking safe or meaningful (DNS, TLS identity, service discovery, firewalling, per-app permission, bandwidth limits) lives in separate ambient subsystems the socket layer neither knows nor enforces. The OS enforces reachability, never *which peer* a given component may talk to, and cannot answer "which services may this app reach, over which protocols, under whose identity, within what budget?" because the socket abstraction discards exactly that.

## The Cathedral Model

Split the problem the way [[ipc_and_service_invocation]] splits IPC: a small privileged substrate the OS owns, and the protocol logic as a library the component links.

**The OS does not own the transport.** TCP, QUIC, TLS, and congestion control are userspace libraries a component links, not a kernel service. This is the exokernel / kernel-bypass direction (DPDK, AF_XDP, Demikernel), and QUIC already proved transport can live in userspace (it runs over UDP, with the connection logic in the app).

**The OS owns demux, authority, and arbitration.** The NIC is one shared device, so something privileged must still:

- Run the NIC driver ([[driver_model]]) and hand each component its own send/receive queues (hardware does this with multiqueue, SR-IOV, and flow steering),
- Demultiplex inbound packets to the right owner,
- Enforce isolation, so a component cannot send as, or read the traffic of, another,
- Arbitrate the scarce shared resources: addresses, ports, bandwidth, queues.

**A network capability is a flow authorization.** It is not "can do network," and not a syscall into a kernel stack. It is the right to exchange packets with a named, identity-verified peer, enforced where packets meet the shared NIC: the broker installs flow-steering rules only for the peers a component's capability permits, and the NIC drops everything else. So a component runs its own transport over an authorized flow, but the *set of peers it can reach* is gated by the privileged demux, not by its own untrusted stack. This is what resolves the obvious objection that a userspace stack can craft a packet to any address: it can, and the NIC drops it.

```omega
// The OS broker authorizes a flow to a named, identity-verified peer.
// It resolves the name, pins the peer identity, and installs the NIC flow rule.
let flow = net.authorize(
    peer = Service("api.vendor.com"),   // a name, not 34.117.12.9:443
    identity = VendorIdentity,          // pinned, see [[identity_and_principals]]
    cap = reach_payments,               // Capability<Network::Reach<...>>
);

// Transport is a library over the authorized flow. The component owns it.
let conn = Quic::open(flow, PaymentAPI::v3);
```

The component never names `34.117.12.9:443`; it names a service, and authority is per-peer, so revoking "may reach this service" is a precise graph operation, not a firewall-rule edit. Because the authorized flow can carry a typed protocol, the resulting endpoint is identical in shape to a local IPC endpoint ([[ipc_and_service_invocation]]): the same `wire data` schema and capability passing work whether the peer is local or across the network. A component that wants raw authorized packets can take the flow directly and skip the typed layer.

## Concerns & Design Space

- **Demux and flow rules.** How a peer is authorized at the device: hardware flow steering / SR-IOV when the NIC supports it (the broker just configures it), a software demux when it does not, and what that software path costs.
- **Naming, DNS & discovery.** Resolution is a brokered, auditable step that yields a verified peer, not a bare address; discovery is local-first and registry-backed ([[naming_and_discovery]]).
- **Peer identity.** Connections bind to identities on both ends ([[identity_and_principals]]); pinning and attestation are part of `authorize`, not bolted on after `connect`.
- **Firewall as capability ceilings.** Egress/ingress policy is a ceiling over the network capability graph, not a parallel ambient ruleset.
- **Bandwidth & budgets.** Network is a metered resource; budgets are accounted via effects and governed by the scheduler ([[scheduler_and_resources]]). Congestion fairness may have to be enforced at the demux even though the transport is a library (see below).
- **Transport as a library.** QUIC, TLS, multipath, connection migration, and NAT traversal are libraries over an authorized flow, so mobility and multihoming are normal rather than heroic, and the choice of stack is the component's.
- **Observability.** Every flow is attributable by construction: which principal, which peer identity, which protocol, how much bandwidth ([[observability_and_introspection]]).

## Key Questions

- What is the minimal demux/broker, and how does it stay small enough to be TCB-worthy while resolving names, pinning identity, and installing flow rules?
- Who enforces congestion-control fairness when each component owns its own transport? A misbehaving userspace stack can be antisocial, so does the demux or NIC enforce pacing regardless ([[scheduler_and_resources]])?
- How much of demux and flow authorization can be offloaded to a smartNIC/DPU, leaving the OS as a configurator, versus done in a software fast path?
- Where do bandwidth budgets live (per principal, per capability, per flow), and how do they compose?

## Omega Leverage

- **Capabilities as values** make a flow authorization a held, attenuable, revocable grant rather than an ambient socket right. See Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **`effects` (`network_io`)** mark and account every crossing into the network, giving the boundary edge where the OS broker provides authority.
- **`wire data`** frames the typed-library protocol with stable field numbers and compatibility rules, identical to local IPC, so cross-version interop and compatibility reports apply to the network too. See Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **`boundary` providers** are the home for the NIC driver below the demux and for the transport stack (TLS, QUIC, NAT traversal) below the connection API; the demux/broker itself is Omega, keeping that TCB small and checked.
- Omega does not yet model bandwidth as a metered effect or peer attestation as a fact; both are extensions Cathedral drives.

## Open Questions

- How do per-peer, identity-bound capabilities interoperate with the existing internet, where most peers speak neither Cathedral protocols nor capability identity? A legacy transport library reaching a raw `IP:port` under a coarse capability is the likely bridge.
- Does moving transport into N per-app libraries trade one kernel-stack attack surface for N library ones, and is that net positive given each is sandboxed and individually patchable?
- Is raw packet access purely a driver-level concern ([[driver_model]]), or an attenuated escape hatch some components can hold?
- What is the offline contract: which capabilities stay usable, and how does budget accounting behave under partition ([[distributed_boundary]])?

## Related
- [[capability_model]] — a flow authorization is a capability in the graph.
- [[driver_model]] — the NIC is a device with a driver under the demux.
- [[ipc_and_service_invocation]] — an authorized flow carrying a typed protocol is a protocol endpoint.
- [[distributed_boundary]] — networking as the path off the machine.
- [[identity_and_principals]] — peer identity binds both ends of a flow.
- [[scheduler_and_resources]] — bandwidth as a governed budget, and congestion fairness.
