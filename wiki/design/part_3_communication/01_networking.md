# Chapter 01: Networking

> Network access is a per-destination capability: the right to exchange packets with a named, identity-verified peer, enforced where packets meet the shared Network Interface Controller (NIC). The OS owns that demux and the authority; the transport (TCP, QUIC, TLS) is a userspace library.

## The Legacy Model

In a traditional OS, networking bottoms out at the socket: a process opens a file descriptor to an IP address and port and is then free to send and receive arbitrary bytes. Two things are wrong. First, the kernel owns one general-purpose TCP/IP stack that every app shares, which fixes the policy (its congestion control, its buffering, its feature set) and costs a syscall on every operation. Second, authority is ambient and binary: an app that can open a socket can reach anything the routing table allows. Everything that makes networking safe or meaningful (DNS, TLS identity, service discovery, firewalling, per-app permission, bandwidth limits) lives in separate ambient subsystems the socket layer neither knows nor enforces. The OS enforces reachability, never which peer a given component may talk to. It cannot answer "which services may this app reach, over which protocols, under whose identity, within what budget?" because the socket abstraction discards exactly that.

## The Cathedral Model

Cathedral splits networking the way [[ipc_and_service_invocation]] splits IPC: a small privileged substrate the OS owns, and the protocol logic as a library the component links.

### The OS does not own the transport

TCP, QUIC, TLS, and congestion control are userspace libraries a component links, not a kernel service. This is the exokernel / kernel-bypass direction (DPDK, AF_XDP, Demikernel). QUIC already proved transport can live in userspace: it runs over UDP, with the connection logic in the app.

### The OS owns demux, authority, and arbitration

The NIC is one shared device, so something privileged must still:

- Run the NIC driver ([[driver_model]]) and hand each component its own send/receive queues. Hardware does this with multiqueue, SR-IOV, and flow steering.
- Demultiplex inbound packets to the right owner.
- Enforce isolation, so a component cannot send as, or read the traffic of, another.
- Arbitrate the scarce shared resources: bandwidth, NIC queues, and flow-table capacity. `IP:port` addressing is a legacy-internet bridge detail, not a Cathedral primitive it hands out.

### A network capability is a flow authorization

A **network capability** is the right to exchange packets with a named, identity-verified peer, enforced where packets meet the shared NIC. It is not "can do network," and not a syscall into a kernel stack. The broker installs flow-steering rules only for the peers a component's capability permits, and the NIC drops everything else. A component runs its own transport over an authorized flow, but the set of peers it can reach is gated by the privileged demux, not by its own untrusted stack. This answers the obvious objection that a userspace stack can craft a packet to any address. It can, and the NIC drops it.

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

The component never names `34.117.12.9:443`. It names a service, and authority is per-peer, so revoking "may reach this service" is a graph operation, not a firewall-rule edit. Because the authorized flow can carry a typed protocol, the resulting endpoint is identical in shape to a local IPC endpoint ([[ipc_and_service_invocation]]): the same numbered schema, selected codec, and capability passing work whether the peer is local or across the network. A component that wants raw authorized packets can take the flow directly and skip the typed layer.

## Recursive networking

The network provider is a recursive interface, the same shape as the compositor ([[windowing_and_compositor]]). Its operations are resolve a name, authorize a flow to a peer, accept inbound flows, and carry packets. Any component that holds network authority can implement that interface for its children and bind their network resolution to its own endpoint at spawn. The child resolves "the network" from its environment and opens flows exactly as it would against the OS broker, and cannot tell whether the other end is the real demux or a parent. That one pattern is a VPN, a firewall, a proxy, a NAT, container networking, a virtual machine's NIC, and a fully fabricated network for tests.

Several properties follow:

- **Egress control is attenuation.** A parent can delegate only the reach it holds, so a child's set of reachable peers is bounded by what its parent passed down, and reach narrows as you descend the tree. Per-app egress filtering is the capability algebra ([[capability_model]]), not a separate ruleset.
- **Naming nests.** Resolution is part of the interface, so the child's name-to-peer map is whatever the parent serves: split-horizon, blocked hosts, fabricated peers. This is the per-principal resolution environment ([[filesystem_as_database]]) pointed at network endpoints.
- **Inbound nests.** A parent can mediate which inbound flows reach a child and publish the child under the parent's own address. That is a reverse proxy or an ingress, the listen-side dual.
- **Proxy or synthesize.** A nested provider either holds a flow to its own real network and forwards after policy (NAT, firewall, proxy), or terminates the flow at itself.

The termination case deserves a name. A parent can authorize a child's flow to some peer and answer it locally instead of routing to a wire, so the child believes it reached `api.vendor.com` while it is talking to the parent. A locally terminated flow lowers to the shared-region IPC primitive ([[ipc_and_service_invocation]]), so local and remote are two lowerings of one endpoint, and this is the parent choosing the local lowering and serving the call itself. Terminating a child's flow at the provider is a clean way to emulate a server for tests, terminate cloud calls locally for offline or local-first operation, virtualize a service, record and replay, inject faults, shim a dead backend, or sandbox an app behind a fake internet that never touches a NIC. Two caveats apply. Termination is free, but being a convincing server means implementing enough of the wire protocol the child expects. And a TLS peer requires a certificate the child will accept.

The man-in-the-middle position is the real difference from display and audio. Those terminate at hardware you own, so a nesting parent is benign. The network terminates at a remote, so a nesting parent sits in the data path, which is a man in the middle. How much that costs the child is a crypto gradient governed by who controls its trust anchors. With no encryption, or when the child trusts a trust-anchor set the parent supplied, the parent reads and rewrites everything and can mint a certificate for any peer, which is how an intercepting proxy with an installed root works. When the child pins a baked-in peer key, the parent drops to a blind carrier that sees metadata and can drop or delay but cannot read or forge. The invariant that survives network nesting is end-to-end authentication the child performs itself, because the parent legitimately carries the bytes and there is nothing for the OS to draw over. Capabilities bound what the child can reach. The child's own crypto bounds content and identity against its carrier. The authority graph keeps "who is my network provider" a recorded fact ([[observability_and_introspection]]), so the man in the middle is always named even when it is allowed.

## The settled mechanism (networking audit)

This section holds the mechanism that came out of the networking audit: the port split, the two-capability session, per-Matrix loopback, egress enforcement by peer class, the legacy bridge, the exfiltration ceiling, inbound reachability, congestion fairness, QUIC routing, the open-web client, legacy public ingress, and hardware steering slots. The residuals that remain open (first-introduction and first-pin, relay-commons economics, and recovery without a backdoor) are tracked in [networking_audit_brief.md](networking_audit_brief.md).

### Ports are deleted locally and demoted on the wire

Inside the machine there is no listening integer. A service is its endpoint capability, and "bind" means register an endpoint. A port's three fused jobs split: demux is the broker's steering job, identity is the pinned peer key, and authority is the held flow capability. The whole bug class is uninstantiable: no `EADDRINUSE`, no privileged `<1024`, no `SO_REUSEPORT` theft, no ephemeral-port exhaustion between local peers. One thing is forced. The wire still carries a 16-bit port to legacy peers, and a public Cathedral server must listen on a fixed port (443/53/22) and demux inward after accept. The number a stranger dials runs before any Cathedral logic, so capability routing governs only the post-accept path.

### Send and receive are two capabilities; the session is bidirectional

Egress is `Reach<Peer>`, the right to open a flow toward a peer. Ingress is `Serve`, the right to be reachable as an endpoint key and accept inbound. They are independently grantable, attenuable, and revocable. Once a flow establishes it is bidirectional, like local IPC. One thing is forced. The split is an authority distinction, not an operational one. Behind NAT/CGNAT a pure server holding only `Serve` still needs `Reach` to a relay or rendezvous plus keepalive to be inbound-reachable at all. Standalone serve-with-zero-reach exists only with a public address or an IPv6 pinhole.

### Loopback is per-Matrix and recursion-free

There is no ambient `127.0.0.1` a native component can guess. A same-machine service is a named, capability-held endpoint resolved from the Matrix's own resolution environment. The Matrix asks for its own loopback and never rethrows to the outer Matrix. The localhost-daemon disaster class (CSRF-to-localhost, port-squatting, `loopback == trusted`) is uninstantiable for native code. The scope is narrower for foreign code. A legacy/POSIX compatibility box reproduces ambient `127.0.0.0/8` and integer ports faithfully inside itself for the foreign code that demands them, so the contribution there is blast-radius confinement, not elimination.

### Egress enforcement quality depends on peer class

Egress enforcement is not uniform. A Cathedral-native peer pinned by public key is enforced end-to-end and survives IP changes. That is the strong case. The NIC's hardware floor is the **5-tuple** (source/dest IP, source/dest port, protocol); key-level identity is a software check, post-handshake, not silicon. A legacy CDN-fronted peer is the weak case. One IP fronts ~50k tenants and **ECH** (Encrypted Client Hello) hides the **SNI** (the requested hostname), so "the NIC drops everything else" authorizes the whole shared front and true per-peer enforcement is impossible there.

The bridge to the existing internet, where most peers speak neither Cathedral protocols nor capability identity, is a legacy transport library reaching a raw `IP:port` under a coarse capability. Its limits are the ones above: at the CDN/ECH edge a legacy grant authorizes the shared front, not one site. The native wins scale only with the Cathedral-native and store-mediated fraction of a user's peers, which is near zero at launch. See [networking_audit_brief.md](networking_audit_brief.md).

### Exfiltration has a hard ceiling

Capabilities bound the peer set, not the bytes. Reaching an arbitrary endpoint is closed: no flow capability, no reach. DNS-name smuggling (encoding data into lookup names) is partially closed. Ambient resolution is gone, but an app reaching an authorized resolver can still emit attacker-influenced query names, and only fully post-DNS, key-only provisioning closes it. Tunnelling stolen data inside an already-authorized flow is not closeable at all. This is the permanent ceiling of the whole capability model, and the only structural mitigation is a **conjunction ceiling** denying the dangerous combination (for example, `Read<Photo>` plus `Network` reach). Metadata always leaks: pinning hides content and identity, never the existence, timing, or social graph of a flow.

### Inbound reachability is not the local OS's to solve

Outbound works nearly everywhere. Being reachable requires a public address that routes to you, which behind NAT/CGNAT lives in upstream middleboxes the OS does not own and cannot conjure. The OS runs the shipping playbook as a library in the recursive network provider: ICE candidate gathering (host/STUN/relay), happy-eyeballs across IPv4-NAT and IPv6, and relay-to-direct upgrade. It prefers IPv6 but never assumes it, and falls back to a **relay** when direct fails. The relay is unavoidable, since with no direct path someone with a public address must carry the bytes. It can be made untrusted (end-to-end encrypted, so it sees ciphertext and metadata only) but never free. On CGNAT there is no port-mapping to grant at all, so the default is relay, not a port-forward. The OS's positive levers on the IPv4/IPv6 split are to prefer IPv6, make it unlock the direct un-relayed path, and make the IPv4/CGNAT tax legible. It never punishes IPv4 users.

### Congestion fairness is the scheduler's job

Each component owns its own transport, so a misbehaving userspace stack can be antisocial. The scheduler does per-world fair-queuing over a paced `EgressBudget` ([[scheduler_and_resources]]). Cathedral forces its own worlds fair regardless of each stack's congestion control. It cannot force the off-machine bottleneck.

### QUIC routing and the classifier discipline

Inbound routing is asymmetric. A packet arrives addressed to no one yet, so something trusted must classify it before its owner is known. The worlds' own arbitrary, untrusted transport libraries cannot, because you do not yet know which world to ask. The demux must therefore read a routing token itself. For QUIC that token is the **Connection ID**, left in the cleartext invariant header (RFC 8999) so a router can steer without decrypting. The demux reads the CID and steers. It never parses the encrypted payload, terminates TLS, or understands streams: route on the token, never on semantic state.

The broker mints and partitions the token space. A CID is endpoint-chosen, so if each world's arbitrary stack picked CIDs freely, two worlds could collide, or a malicious world could pick a CID to claim another world's inbound packets (hijack or eavesdrop), breaking the demux's one provable invariant that a packet lands in the right inbox. The broker hands each world a **partition** of the CID space, a prefix it must embed (the QUIC-LB approach). The world's stack chooses CIDs only within its partition, and the demux routes by the broker-minted prefix. The routing token is broker-issued, capability-like state, never endpoint-chosen, which is the same move as deleting ports, applied to QUIC. The provable cross-delivery prevention holds only for broker-minted CIDs. A stock peer that chose its own CIDs is routed by the 5-tuple instead, so that connection cannot transparently survive the peer's migration. That is the legacy edge again.

The classifier is a loaded module, not kernel baggage and not an ambient service. The demux core is protocol-agnostic and ships knowing nothing about QUIC. A protocol's routing-envelope knowledge (where its token sits) lives in a small **proved classifier module**, dead until first use (first QUIC ingress) and torn down when unused, under Cathedral's dead-until-requested rule with no ambient daemon. Because isolation is proof, not an MMU wall, the classifier loads into the router's domain at hot-path speed with no per-packet IPC, stays isolated by construction, and is hot-swappable as the protocol evolves. Two capabilities are kept apart. `Demux::RegisterClassifier` (teach the router a protocol) is privileged TCB held only by trusted OS networking code; apps never hold it, and a third-party classifier loads only by a high-trust grant. `Network::Serve<…>` (receive on a CID partition) is what an app holds.

Apps address their provider, never the driver. A world resolves "the network" at start as a **provider endpoint** capability: the recursive interface above, meaning its parent or the OS broker. `serve()`/`listen()` is a protocol call over the one IPC primitive (a capability-scoped shared region) to that endpoint: write the request, signal, then `suspend` while the provider mints the partition. A parent provider handles it, proxies it upward (another hop up the Matrix tree, attenuating and observing as it passes, which is egress control for free), or terminates it locally. The chain bottoms out at the OS broker, which owns the single NIC driver (a confined user-mode component below it) and the steering table, mints the partition, installs `prefix → inbox`, and returns it down. "The driver is not in my Matrix" is a non-issue. You never address the driver; you address your provider, the chain reaches the broker, and the broker owns the driver. One physical NIC, one driver, one broker, N cheap provider-endpoints multiplexed by the steering table, and never a driver per Matrix. SR-IOV can hand a hot Matrix a dedicated virtual-function queue for near-direct zero-copy, but routing authority still flows through the broker. That is a performance optimization, not a second driver.

### The open-web client: contain, don't enumerate

A large, central class of clients (a browser, `curl`, `git`, an IDE fetching arbitrary registries) reaches peers that do not exist at grant time and are chosen by the user at runtime. Their peer set cannot be enumerated in a manifest or pinned by key. The wrong reflex is to mint a flow per navigation from the URL bar. The OS does not own the browser's chrome, since the URL bar is the browser's own pixels, so a "the user navigated to X" report is an untrusted claim, not a trusted mint.

The right model is containment, not enumeration: such a client is just a Matrix. Give it full reach to the open internet, because talking to the whole web is its job and bounding its peer set is both impossible and pointless. Give it zero ambient local authority: no files, devices, secrets, or other-app reach. Full net is then harmless because the sandbox holds nothing worth exfiltrating. This is the conjunction ceiling in its canonical form. A principal with unrestricted `Network` reach must hold no `Read` over anything sensitive, and the browser is the instance that makes the rule concrete. The only inbound bridge is the **picker** (pick a file and it gets that one) plus scoped downloads. Nothing flows in ambiently.

Two notes. "Empty" means empty of ambient authority. The client still holds what the user actively puts into it this session (a typed password, a picked file, a site's own login) and can leak those. Containment bounds the blast radius to what this instance was handed, never the whole machine, but not to zero. The cheap mitigation is **per-trust-domain instances**: spinning up a throwaway browser-Matrix for a sketchy link and a separate one for banking is trivial here, replacing today's one browser that holds everything. Legacy tools reach this model through a POSIX shim that maps `connect(ip:port)` onto a flow authorization against the resolution environment. The inner dev loop gets a developer-Matrix whose owner holds broad reach over its own realm, confined and non-leaking, so `localhost:3000`-style work needs no per-connection prompt.

### Legacy public ingress: the exposed front-end

The clean ingress path is reachable-as-key: a published rendezvous and a pinned-key handshake gate the first inbound packet, so no stranger gets through. But a public website must answer stock clients, a browser or `curl` that does DNS, then a SYN to a fixed port (`:443`), then a TLS handshake with no concept of a key. There is no capability to check, because the stranger holds none and "anyone may connect" is the purpose. "Default-deny = absence of a grant" is therefore an internal-namespace property. It governs which inner world a flow reaches, never whether a stranger may attempt a public connection.

Public legacy ingress is a carved-out exception to the demux's "never parse/terminate" rule, isolated as its own minimal component:

- **A public-ingress front-end** (not the core demux) accepts the unauthenticated TCP/TLS connection, reads the **ClientHello SNI** (the requested hostname), and steers it to the world that serves that name. This is the recursive "inbound nests" case made concrete, a reverse proxy or SNI router. It prefers **SNI-passthrough**: it routes by name and never terminates TLS or sees plaintext, so each served world keeps its own certificate and key. Under ECH, which encrypts the SNI, the front-end holds only the operator's scoped ECH key to read the inner name, never the site's TLS key, the way a CDN does it today.
- **Dead by default.** A machine that hosts no public service runs no front-end and has zero stranger-facing open ports. That is a major attack-surface win over today, where localhost daemons and bound services litter every machine. The front-end exists only when a world holds `Serve<Public>`, a distinct, higher-authority "be reachable by unauthenticated strangers on a well-known port" grant. It is strictly broader than native `Serve<Key>`, most apps never hold it, and a web server requests it.
- **DoS is inherent and not eliminable.** Answering strangers is attackable. The front-end eats SYN floods and unauthenticated-connection floods exactly like `nginx` today, mitigated by the standard means: SYN cookies, connection rate-limits, the scheduler's per-front-end fair-queue budget, and upstream anycast/scrubbing at serious scale. Cathedral's contribution is isolation and attribution, not elimination of the exposure. A front-end compromise or overload is contained to its own Matrix and cannot reach the served worlds or the system, and every accepted connection is steered into a named world and observable. The clean reachable-as-key path is the default; the exposed front-end is the opt-in, isolated cost of serving the legacy public web.

### Hardware steering slots as a scheduled capability

The NIC's flow-steering table, the hardware entries that deliver a flow's packets straight to its owner's queue (the fast path, zero-copy-capable), is finite, so the fast path is a scarce shared resource. It is not handed out by ambient broker discretion, since a greedy world would grab every entry and starve the rest onto the software demux path. It is a held, budgeted capability.

That capability is the right to be scheduled into the pool, not ownership of a slot. The broker places a flow into a hardware entry opportunistically when there is room and evicts it under pressure, exactly as the scheduler time-shares any scarce resource ([[scheduler_and_resources]]). A flow without a current entry runs on the software path. No flow hoards an entry, and the fast path is fair-shared rather than first-come-claimed.

Allocation status is not exposed, which closes the obvious side-channel. The broker never returns "you got hardware" versus "you fell back to software"; a holder only knows it holds the right, so a world cannot probe global pool occupancy through the allocation interface. The residual is a timing channel. A flow still experiences its own latency, and fast-versus-software is the bit, so a world measuring its own packet timing can in principle infer aggregate pool pressure. That residual is irreducible, because the slot's whole value is a latency effect the beneficiary necessarily observes, but low-SNR in practice: the hardware-versus-software demux difference is microseconds, swamped by millisecond network jitter, so it is negligible for real network flows. Two tiers result. **Opaque dynamic scheduling** is the default: maximum utilization, the allocation channel closed, only the weak timing residual. A **partitioned or reserved sub-pool** is the paranoid or local-high-precision opt-in: a flow's latency then depends only on its own budget, closing even the timing residual, at the cost of idle reserved entries.

## Concerns & Design Space

- **Demux and flow rules.** How a peer is authorized at the device: hardware flow steering / SR-IOV when the NIC supports it (the broker configures it), a software demux when it does not, and what that software path costs.
- **Naming, DNS & discovery.** Resolution is a brokered, auditable step that yields a verified peer, not a bare address. Discovery is local-first and registry-backed ([[naming_and_discovery]]).
- **Peer identity.** Connections bind to identities on both ends ([[identity_and_principals]]). Pinning and attestation are part of `authorize`, not bolted on after `connect`.
- **Firewall as capability ceilings.** Egress/ingress policy is a ceiling over the network capability graph, not a parallel ambient ruleset.
- **Bandwidth & budgets.** Network is a metered resource. Reach makes use visible, while budget capabilities and provider accounting enforce quantity under scheduler policy ([[scheduler_and_resources]]). Congestion fairness is enforced by the scheduler's per-world fair-queuing even though the transport is a library (see above).
- **Transport as a library.** QUIC, TLS, multipath, connection migration, and NAT traversal are libraries over an authorized flow, so mobility and multihoming are normal cases the model handles, and the choice of stack is the component's.
- **Observability.** Every flow is attributable by construction: which principal, which peer identity, which protocol, how much bandwidth ([[observability_and_introspection]]).
- **Nested providers.** A component can implement the network interface for its children (VPN, firewall, NAT, a VM's NIC, or terminating flows locally), with reach bounded by attenuation and confidentiality bounded by the child's end-to-end crypto.
- **Zero value.** A zero flow authorization is the null flow reaching no peer (shape 2, inert). It is both least-privilege (the NIC steers nothing to or from it) and ZII-coherent, so a component handed a zeroed flow sends into the void and receives nothing instead of erroring, matching default-deny egress ([[omega_substrate]]).

## Key Questions

- What is the minimal demux/broker, and how does it stay small enough to be TCB-worthy while resolving names, pinning identity, and installing flow rules?
- How much of demux and flow authorization can be offloaded to a smartNIC/DPU, leaving the OS as a configurator, versus done in a software fast path?
- Where do bandwidth budgets live (per principal, per capability, per flow), and how do they compose?
- When a parent terminates a child's flow locally instead of routing it, should the OS surface that the flow did not leave the machine, given the child cannot otherwise tell, and what stops a malicious provider impersonating a sensitive peer beyond the child pinning identity?

## Omega Leverage

- **Capabilities as values** make a flow authorization a held, attenuable, revocable grant rather than an ambient socket right. See Omega [Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **`reaches Network`** marks and accounts every crossing into the network, naming the boundary edge where the OS broker provides authority.
- **Selected wire codecs over ordinary numbered schemas** frame the typed-library protocol with stable identities and compatibility rules, identical to local IPC, so cross-version interop and compatibility reports apply to the network too. See Omega [Wire Protocols](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#wire-protocols).
- **`boundary` providers** are the home for the NIC driver below the demux and for the transport stack (TLS, QUIC, NAT traversal) below the connection API. The demux/broker itself is Omega, keeping that TCB small and checked.
- **The network provider is a trait** any component can implement, so nesting is one interface with many implementations resolved from the child's environment. A nested provider is that resolution bound to a parent endpoint. See Omega [Traits](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md).
- Omega does not yet model bandwidth as a metered effect or peer attestation as a fact. Both are extensions Cathedral drives.

## Open Questions

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
- [[filesystem_as_database]] — the per-principal resolution environment that nested naming reuses.
- [[windowing_and_compositor]] — the recursive-provider pattern networking mirrors.
