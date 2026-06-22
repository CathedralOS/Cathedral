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
- Arbitrate the scarce shared resources: bandwidth, NIC queues, and flow-table capacity. (`IP:port` addressing is a legacy-internet bridge detail, not a Cathedral primitive it hands out.)

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

## Recursive networking

The network provider is a recursive interface, the same shape as the compositor ([[windowing_and_compositor]]). Its operations are resolve a name, authorize a flow to a peer, accept inbound flows, and carry packets. Any component that holds network authority can implement that interface for its children and bind their network resolution to its own endpoint at spawn. The child resolves "the network" from its environment and opens flows exactly as it would against the OS broker, and cannot tell whether the other end is the real demux or a parent. That one pattern is a VPN, a firewall, a proxy, a NAT, container networking, a virtual machine's NIC, and a fully fabricated network for tests.

Several properties fall out:

- **Egress control is attenuation.** A parent can delegate only the reach it holds, so a child's set of reachable peers is bounded by what its parent passed down, and reach narrows as you descend the tree. Per-app egress filtering is the capability algebra ([[capability_model]]), not a separate ruleset.
- **Naming nests.** Resolution is part of the interface, so the child's name-to-peer map is whatever the parent serves: split-horizon, blocked hosts, fabricated peers. This is the per-principal resolution environment ([[filesystem_as_database]]) pointed at network endpoints.
- **Inbound nests.** A parent can mediate which inbound flows reach a child and publish the child under the parent's own address, which is a reverse proxy or an ingress, the listen-side dual.
- **Proxy or synthesize.** A nested provider either holds a flow to its own real network and forwards after policy (NAT, firewall, proxy), or terminates the flow at itself.

That termination case is worth naming. A parent can authorize a child's flow to some peer and answer it locally instead of routing to a wire, so the child believes it reached `api.vendor.com` while it is really talking to the parent. Because a locally terminated flow lowers to the shared-region IPC primitive ([[ipc_and_service_invocation]]), local and remote are two lowerings of one endpoint, and this is the parent choosing the local lowering and serving the call itself. Terminating a child's flow at the provider is a clean way to emulate a server for tests, terminate cloud calls locally for offline or local-first operation, virtualize a service, record and replay, inject faults, shim a dead backend, or sandbox an app behind a fake internet that never touches a NIC. Two honest caveats: termination is free, but being a convincing server means actually implementing enough of the wire protocol the child expects, and a TLS peer requires a certificate the child will accept.

The man-in-the-middle position is the real difference from display and audio. Those terminate at hardware you own, so a nesting parent is benign. The network terminates at a remote, so a nesting parent sits in the data path, which is exactly a man in the middle. How much that costs the child is a crypto gradient governed by who controls its trust anchors. With no encryption, or when the child trusts a trust-anchor set the parent supplied, the parent reads and rewrites everything and can mint a certificate for any peer, which is how an intercepting proxy with an installed root works. When the child pins a baked-in peer key, the parent drops to a blind carrier that sees metadata and can drop or delay but cannot read or forge. So the invariant that survives network nesting is end-to-end authentication the child performs itself, because the parent legitimately carries the bytes and there is nothing for the OS to draw over. Capabilities bound what the child can reach, the child's own crypto bounds content and identity against its carrier, and the authority graph keeps "who is my network provider" an explicit fact ([[observability_and_introspection]]), so the man in the middle is always named even when it is allowed.

## The settled mechanism (networking audit)

This consolidates the converged points of the networking audit up to the *ingress* problem. The harder residuals — public legacy ingress, the QUIC demux-token discipline, first-introduction/first-pin, relay-commons economics, and the open-web client tier — are tracked in [networking_audit_brief.md](networking_audit_brief.md) and stay open.

**Ports are deleted locally and demoted on the wire.** Inside the machine there is no listening integer; a service *is* its endpoint capability, and "bind" means *register an endpoint*. A port's three fused jobs split cleanly — **demux** is the broker's steering job, **identity** is the pinned peer key, **authority** is the held flow capability — so the whole bug class is uninstantiable: no `EADDRINUSE`, no privileged-`<1024`, no `SO_REUSEPORT` theft, no ephemeral-port exhaustion between local peers. **Forced:** the wire still carries a 16-bit port to legacy peers, and a public Cathedral server must listen on a fixed port (443/53/22) and demux inward *after accept* — the number a stranger dials runs before any Cathedral logic, so capability routing governs only the post-accept path.

**Send and receive are two capabilities; the session is bidirectional.** Egress is `Reach<Peer>` (open a flow toward a peer); ingress is `Serve` (be reachable as an endpoint key, accept inbound). They are independently grantable, attenuable, and revocable. Once a flow establishes it is bidirectional, like local IPC. **Forced:** the split is an *authority* distinction, not an operational one — behind NAT/CGNAT a pure server holding only `Serve` still needs `Reach` to a relay/rendezvous plus keepalive to be inbound-reachable at all; standalone serve-with-zero-reach exists only with a public address or an IPv6 pinhole.

**Loopback is per-Matrix and recursion-free.** There is no ambient `127.0.0.1` a native component can guess; a same-machine service is a named, capability-held endpoint resolved from the Matrix's *own* resolution environment (it asks for its own loopback, never rethrowing to the outer Matrix). The localhost-daemon disaster class (CSRF-to-localhost, port-squatting, `loopback == trusted`) is uninstantiable for native code. **Scope honestly:** a legacy/POSIX compatibility box reproduces ambient `127.0.0.0/8` and integer ports faithfully *inside itself* for the foreign code that demands them, so the contribution there is blast-radius confinement, not elimination.

**Egress enforcement quality is a function of peer class — it is not uniform.** A Cathedral-native peer pinned by **public key** is enforced end-to-end and survives IP changes (the strong case). The NIC's hardware floor is the **5-tuple** (source/dest IP, source/dest port, protocol); key-level identity is a *software* check, *post-handshake*, not silicon. A legacy CDN-fronted peer is the weak case: one IP fronts ~50k tenants and **ECH** (Encrypted Client Hello) hides the **SNI** (the requested hostname), so "the NIC drops everything else" authorizes the whole shared front and true per-peer is unenforceable.

**Exfiltration has a hard ceiling — capabilities bound the peer set, not the bytes.** Reaching an arbitrary endpoint is *closed* (no flow capability, no reach). DNS-name smuggling (encoding data into lookup names) is *partially* closed (ambient resolution is gone, but an app reaching an authorized resolver can still emit attacker-influenced query names; only fully post-DNS, key-only provisioning closes it). **Tunnelling stolen data inside an already-authorized flow is not closeable at all** — this is the permanent ceiling of the whole capability model, and the only structural mitigation is a **conjunction ceiling** denying the dangerous *combination* (e.g. `deny Read<Photo> ∧ network_io`). And metadata always leaks: pinning hides content and identity, never the existence, timing, or social graph of a flow.

**Inbound reachability is not the local OS's to solve.** Outbound works nearly everywhere; being *reachable* requires a public address that routes to you, which behind NAT/CGNAT lives in upstream middleboxes the OS does not own and cannot conjure. So the OS runs the shipping playbook *as a library in the recursive network provider* — ICE candidate gathering (host/STUN/relay), happy-eyeballs across IPv4-NAT and IPv6, relay→direct upgrade — **prefers IPv6 but never assumes it**, and falls back to a **relay** when direct fails. The relay is unavoidable (no direct path → someone with a public address carries the bytes); it can be made *untrusted* (end-to-end encrypted, ciphertext + metadata only) but never free. On CGNAT there is no port-mapping to grant at all, so the honest default is relay, not a port-forward. The OS's positive levers on the IPv4/IPv6 split are to prefer IPv6, make it *unlock the good (direct, un-relayed) path*, and make the IPv4/CGNAT tax legible — never to punish IPv4 users.

### QUIC routing and the classifier discipline

Inbound routing is asymmetric: a packet arrives addressed to *no one yet*, so something trusted must classify it *before* its owner is known — the worlds' own arbitrary, untrusted transport libraries cannot, because you do not yet know which world to ask. So the demux must read a routing token itself. For QUIC that token is the **Connection ID**, deliberately left in the cleartext invariant header (RFC 8999) precisely so a router can steer without decrypting. The demux reads the CID and steers; it never parses the encrypted payload, terminates TLS, or understands streams — *route on the token, never on semantic state*.

**The discipline: the broker mints and partitions the token space.** A CID is endpoint-chosen, so if each world's arbitrary stack picked CIDs freely, two worlds could collide — or a malicious world could pick a CID to **claim another world's inbound packets** (hijack/eavesdrop), shattering the demux's one provable invariant (a packet lands in the right inbox). So the broker hands each world a **partition** of the CID space (a prefix it must embed — the QUIC-LB approach); the world's stack chooses CIDs only within its partition; the demux routes by the broker-minted prefix. The routing token is **broker-issued, capability-like state, never endpoint-chosen** — the same move as deleting ports, applied to QUIC. The provable cross-delivery prevention holds **only for broker-minted CIDs**; a *stock* peer that chose its own CIDs is routed by the **5-tuple** instead (so that connection cannot transparently survive the peer's migration — the legacy edge again).

**The classifier is a loaded module, not kernel baggage and not an ambient service.** The demux core is **protocol-agnostic** and ships knowing nothing about QUIC. A protocol's routing-envelope knowledge (where its token sits) lives in a small **proved classifier module**, *dead until first use* (first QUIC ingress) and torn down when unused — Cathedral's dead-until-requested rule, no ambient daemon. Because isolation is *proof*, not an MMU wall, the classifier loads **into the router's domain at hot-path speed — no per-packet IPC** — while staying isolated by construction, and is hot-swappable as the protocol evolves. Two capabilities, kept apart: **`Demux::RegisterClassifier`** (teach the router a protocol) is privileged TCB held only by trusted OS networking code — apps never hold it, and a third-party classifier loads only by a deliberate high-trust grant; **`Network::Serve<…>`** (receive on a CID partition) is what an *app* holds.

**Trapping upward: apps address their provider, never the driver.** A world resolves "the network" at spawn as a **provider endpoint** capability (the recursive interface above — its parent, or the OS broker). `serve()`/`listen()` is a protocol call over the one IPC primitive (a capability-scoped shared region) to that endpoint: write the request, signal, await the minted partition. A parent provider handles it, proxies it upward (another hop up the Matrix tree, attenuating/observing as it passes — egress control for free), or terminates it locally. The chain bottoms out at the OS broker, which owns the **single** NIC driver (a confined user-mode component below it) and the steering table, mints the partition, installs `prefix → inbox`, and returns it down. So "the driver is not in my Matrix" is a non-issue — you never address the driver; you address your provider, the chain reaches the broker, the broker owns the driver. **One physical NIC → one driver → one broker → N cheap provider-endpoints**, multiplexed by the steering table; never a driver-per-Matrix. (SR-IOV can hand a hot Matrix a dedicated virtual-function queue for near-direct zero-copy, but routing authority still flows through the broker — a performance optimization, not a second driver.)

### The open-web client: contain, don't enumerate

A large, central class of clients — a browser, `curl`, `git`, an IDE fetching arbitrary registries — reaches peers that do not exist at grant time and are chosen by the user at runtime. Their peer set cannot be enumerated in a manifest or pinned by key. The wrong reflex is to mint a flow per navigation from the URL bar: **the OS does not own the browser's chrome** (the URL bar is the browser's own pixels), so a "the user navigated to X" report is an *untrusted claim*, not a trusted mint.

The right model is **containment, not enumeration**: such a client is *just a Matrix*. Give it **full reach to the open internet** — talking to the whole web *is* its job, and bounding its peer set is both impossible and pointless — and give it **zero ambient local authority** (no files, devices, secrets, or other-app reach). Full net is then harmless because the sandbox holds *nothing worth exfiltrating*. This is the **conjunction ceiling** in its canonical form: a principal holding `network_io` must hold no `Read` over anything sensitive, and the browser is the instance that makes the rule concrete. The only inbound bridge is the **picker** (pick a file → it gets that one) plus scoped downloads; nothing flows in ambiently.

Two honest notes. "Empty" means empty of *ambient* authority — the client still holds **what the user actively puts into it this session** (a typed password, a picked file, a site's own login), and can leak *those*; containment bounds the blast radius to "what this instance was handed," never the whole machine, but not to zero. And the cheap mitigation is **per-trust-domain instances**: spinning up a throwaway browser-Matrix for a sketchy link and a separate one for banking is trivial here, replacing today's one-browser-holds-everything. (Legacy tools reach this model through a POSIX shim that maps `connect(ip:port)` onto a flow authorization against the resolution environment; the inner dev loop gets a developer-Matrix whose owner holds broad reach over *its own* realm, confined and non-leaking, so `localhost:3000`-style work needs no per-connection prompt.)

### Legacy public ingress: the exposed front-end

The clean ingress path is reachable-as-key (a published rendezvous and a pinned-key handshake gate the first inbound packet, so no stranger gets through). But a public website must answer **stock clients** — a browser or `curl` that does DNS → SYN to a fixed port (`:443`) and a TLS handshake with *no concept of a key*. There is no capability to check, because the stranger holds none and "anyone may connect" is the literal purpose. So "default-deny = absence of a grant" is an *internal-namespace* property — it governs which inner world a flow reaches, never whether a stranger may *attempt* a public connection.

So public legacy ingress is a **carved-out exception** to the demux's "never parse/terminate" rule, isolated as its own minimal component:

- **A public-ingress front-end** (not the core demux) accepts the unauthenticated TCP/TLS connection, reads the **ClientHello SNI** (the requested hostname), and steers it to the world that serves that name — the recursive "inbound nests" case made concrete (a reverse-proxy / SNI-router). It prefers **SNI-passthrough**: it routes by name and never terminates TLS or sees plaintext, so each served world keeps its own certificate and key. (Under ECH, which encrypts the SNI, the front-end holds only the operator's scoped *ECH* key to read the inner name — never the site's TLS key, the way a CDN does it today.)
- **Dead by default.** A machine that hosts no public service runs **no front-end and has zero stranger-facing open ports** — a major attack-surface win over today, where localhost daemons and bound services litter every machine. The front-end exists only when a world holds **`Serve<Public>`** — a distinct, higher-authority "be reachable by unauthenticated strangers on a well-known port" grant, strictly broader than native `Serve<Key>`, that most apps never hold and a web server explicitly requests.
- **DoS is inherent and not eliminable.** Answering strangers is inherently attackable: the front-end eats SYN floods and unauthenticated-connection floods exactly like `nginx` today, mitigated by the standard means (SYN cookies, connection rate-limits, the scheduler's per-front-end fair-queue budget, and upstream anycast/scrubbing at serious scale). Cathedral's contribution is **isolation** — a front-end compromise or overload is contained to its own Matrix and cannot reach the served worlds or the system — and **attribution** — every accepted connection is steered into a named world and observable — **not** the elimination of the exposure. The honest line: the clean reachable-as-key path is the default; the exposed front-end is the explicit, opt-in, isolated cost of serving the legacy public web.

## Concerns & Design Space

- **Demux and flow rules.** How a peer is authorized at the device: hardware flow steering / SR-IOV when the NIC supports it (the broker just configures it), a software demux when it does not, and what that software path costs.
- **Naming, DNS & discovery.** Resolution is a brokered, auditable step that yields a verified peer, not a bare address; discovery is local-first and registry-backed ([[naming_and_discovery]]).
- **Peer identity.** Connections bind to identities on both ends ([[identity_and_principals]]); pinning and attestation are part of `authorize`, not bolted on after `connect`.
- **Firewall as capability ceilings.** Egress/ingress policy is a ceiling over the network capability graph, not a parallel ambient ruleset.
- **Bandwidth & budgets.** Network is a metered resource; budgets are accounted via effects and governed by the scheduler ([[scheduler_and_resources]]). Congestion fairness may have to be enforced at the demux even though the transport is a library (see below).
- **Transport as a library.** QUIC, TLS, multipath, connection migration, and NAT traversal are libraries over an authorized flow, so mobility and multihoming are normal cases the model handles, and the choice of stack is the component's.
- **Observability.** Every flow is attributable by construction: which principal, which peer identity, which protocol, how much bandwidth ([[observability_and_introspection]]).
- **Nested providers.** A component can implement the network interface for its children (VPN, firewall, NAT, a VM's NIC, or terminating flows locally), with reach bounded by attenuation and confidentiality bounded by the child's end-to-end crypto.
- **Zero value.** A zero flow authorization is the null flow reaching no peer (shape 2, inert): it is simultaneously least-privilege (the NIC steers nothing to or from it) and ZII-coherent, so a component handed a zeroed flow sends into the void and receives nothing instead of erroring, matching default-deny egress ([[omega_substrate]]).

## Key Questions

- What is the minimal demux/broker, and how does it stay small enough to be TCB-worthy while resolving names, pinning identity, and installing flow rules?
- Who enforces congestion-control fairness when each component owns its own transport? A misbehaving userspace stack can be antisocial, so does the demux or NIC enforce pacing regardless ([[scheduler_and_resources]])? *(Decided: the scheduler does per-world fair-queuing over a paced `EgressBudget` — Cathedral forces its *own* worlds fair regardless of each stack's congestion control; it cannot force the off-machine bottleneck.)*
- How much of demux and flow authorization can be offloaded to a smartNIC/DPU, leaving the OS as a configurator, versus done in a software fast path?
- Where do bandwidth budgets live (per principal, per capability, per flow), and how do they compose?
- When a parent terminates a child's flow locally instead of routing it, should the OS surface that the flow did not leave the machine, given the child cannot otherwise tell, and what stops a malicious provider impersonating a sensitive peer beyond the child pinning identity?

## Omega Leverage

- **Capabilities as values** make a flow authorization a held, attenuable, revocable grant rather than an ambient socket right. See Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **`effects` (`network_io`)** mark and account every crossing into the network, giving the boundary edge where the OS broker provides authority.
- **`wire data`** frames the typed-library protocol with stable field numbers and compatibility rules, identical to local IPC, so cross-version interop and compatibility reports apply to the network too. See Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **`boundary` providers** are the home for the NIC driver below the demux and for the transport stack (TLS, QUIC, NAT traversal) below the connection API; the demux/broker itself is Omega, keeping that TCB small and checked.
- The network provider is a **trait** any component can implement, so nesting is one interface with many implementations resolved from the child's environment; a nested provider is just that resolution bound to a parent endpoint. See Omega [Traits](../../../../Omega/wiki/language_guide/chapter_13_traits.md).
- Omega does not yet model bandwidth as a metered effect or peer attestation as a fact; both are extensions Cathedral drives.

## Open Questions

- How do per-peer, identity-bound capabilities interoperate with the existing internet, where most peers speak neither Cathedral protocols nor capability identity? A legacy transport library reaching a raw `IP:port` under a coarse capability is the likely bridge. *(Audit: yes, that is the bridge — with honest limits: at the CDN/ECH edge per-peer is unenforceable, so a legacy grant authorizes the shared front, not one site, and the native wins scale only with the Cathedral-native + store-mediated fraction of a user's peers, near-zero at launch. See [networking_audit_brief.md](networking_audit_brief.md).)*
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
