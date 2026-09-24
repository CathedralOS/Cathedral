# Chapter 02: Identity, Principals & Trust

> Capabilities answer what may be done. This chapter owns who holds them: the principals that are the nodes of the authority graph, and why we trust them.

## The Legacy Model

Unix models identity as a small integer. A `uid` and a few `gid`s decide access, and authority is ambient over that identity: you can act because of who you are, not what you hold. The model has little vocabulary for the identities that carry the most weight. Publisher identity and component provenance are not represented at the OS level. There is no OS-level notion of an app, a publisher, a component build, a device, an organization, or a session. Those are approximated with usernames, service accounts, signing certificates checked at install time, and MDM profiles layered on top. "Is this really the vendor who shipped the last update?" and "is this the same device that enrolled last week?" are answered outside the OS, if at all. Identity rotation means provisioning a new account and migrating files by hand.

## The Cathedral Model

A **principal** is any entity the authority graph can name as a holder of authority. Cathedral admits many kinds and treats them uniformly as graph nodes: human users, sessions, apps, individual components, package builds, publishers, devices, organizations, services, and remote principals across a trust boundary. Each principal has a stable, unforgeable identity with recorded **provenance**: how the identity was established and who vouches for it. That identity is what edges in the graph attach to (see [[capability_model]]).

The local human username is the least interesting of these. For a platform that distributes software, **publisher identity** and **component provenance** carry far more weight. The binding from a running component back to a signed build back to an attested publisher is what real trust decisions rest on.

```omega
data Principal {
    kind: PrincipalKind;     // User | App | Component | Publisher | Device | Org | Service | Session | Remote
    id: PrincipalId;         // stable, unforgeable
    provenance: Attestation; // who vouches, and how it was established
}

domain Principal::Attested
    requires self.provenance in Attestation::Verified;
```

### The decided mechanism: identity is the confined-world primitive

A principal is a **confined world**, the same primitive as a sandbox, a filesystem realm, a tenant, or the legacy box. "User" is not special. A user is a confined world that root mints, and identity nests with one primitive: bare machine, then user, then app, then tab. This is the recursive-provider pattern ([[capability_model]]), and it generalizes the nested-principals point below to every level.

One primitive collapses the trust-chain roots into one anchored chain:

- **Root is the measured bare machine.** It is the principal the hardware root of trust and measured boot vouch for ([[boot_and_trust_chain]]), and it owns the login program and the bare services.
- **Root mints the user-worlds and grants the first capabilities into them.** Minting makes it the root of identity. Granting makes it the root of authority. The two roots are one. The login program is a root-owned chooser: logging in selects and unseals a world and runs the session inside it.
- **A user-world's identity is its sealed realm plus the credential that unseals it** ([[sessions_and_login]]).

The chain reads: hardware root, measured-good boot, root (the authority and identity root), user-worlds, app-worlds. The root of identity lives in root, and root mints every principal.

Consequences:

- **"Run as root" is essentially not a thing.** Root is minimized to login plus bare services and is never inhabited by user-facing software. It is the minimized single point of total compromise; you cannot cage the cage ([[kernel_architecture]]). Only a bare server case skips the login program and runs a world directly.
- **Confinement is informational, not only authoritative.** A world knows nothing outside itself by default: no enumeration of sibling worlds, no shared filesystem namespace. "Browse every account's files" is a normalized ambient leak, and it is gone. The only entity that knows the list of worlds is root, and it exposes that list only to the login program.
- **Cross-world sharing is cross-machine sharing.** A world is opaque from outside, so a sibling world is, to you, exactly like a remote host. Sharing is identity-addressed capability-passing: grant a capability to a content-addressed object, addressed to an identity you already hold out-of-band. It is not a shared namespace. This unifies local multi-user with distributed multi-machine ([[distributed_boundary]]). Co-ownership (the family computer) is a **shared world** both identities hold capabilities into. Locally that shared realm is a zero-copy copy of the hierarchy, with storage decoupled and content-addressed, so it is nearly free.
- **Indistinguishability.** Because everything runs in a world, being in one is not suspicious, which defeats sandbox-detect-and-evade malware. The bound is "no reliable tell," not "impossible to tell."
- **Per-persona unlinkability.** A human may hold several unlinkable worlds (work, personal, anonymous). This is the per-relationship-identity point below taken to its conclusion.
- **A user is also a seat, not only a world.** The confined world is the identity and authority half. A **seat**, the input devices currently routed to a principal, is the actor half. "The user," as the thing that acts at the surface, is whoever holds the input ([[windowing_and_compositor]] multi-cursor). There is no special human entity, only a seated principal. Physical and virtual input are indistinguishable to the consuming surface. This is the synthetic-world property: feed a world a virtual seat to drive, test, record, or replay it, the input arm of the synthetic clock, network, and realm. Input is still attributed to the observer, since the compositor labels each seat by its principal, and the physical seat is OS-attested, since the OS alone knows which seat the real devices feed. An agent is a seated principal too ([[agents_as_principals]]). "Let it drive" delegates an input capability, either its own labeled seat or shared access to yours, visible and revocable. An agent holding raw input control is the user; a scoped agent is bounded. The OS key reclaims the physical-device-to-seat binding. That reclaim is hardware-rooted and severs every other consumer, which makes it the unspoofable root of human presence and the one input a software-capability agent cannot supply.

## Concerns & Design Space

- **What an identity is per kind.** A user, an app, a publisher, a device, and a component are not the same shape. Each needs its own answer for what makes two references the same principal, and what survives reinstall, rebuild, or reboot.
- **Caller identity is kernel bookkeeping, not a claim.** A syscall is a trap by a thread the kernel itself scheduled, so "who is calling" is read from the task object: never from an argument, never forgeable, language-irrelevant. Running instances are named by **generational instance ids**, never recyclable pids, so a stale reference to a dead instance is recognized as invalid rather than colliding with its successor. The chain from instance to measured image to manifest is forged at spawn, link by link, all kernel-maintained ([[boot_and_trust_chain]]). That chain is what lets the grant arena check a manifest ceiling at delegation time ([[capability_lifecycle]]).
- **Identity does not propagate through intermediaries.** When A asks service B to act, the kernel sees B, and "on behalf of A" is a recorded delegation edge, not an impersonation claim. This limit is what kills the confused deputy.
- **Provenance over names.** Trust flows from attestation chains (publisher to build to component), not from a string, and the chain is what identifies the principal.
- **Rotation.** Keys and identities must rotate without orphaning everything a principal held. This is a graph-rewrite problem, not a re-provisioning one, and it leans on key management ([[secrets_and_keys]]).
- **Compromised publishers.** When a publisher's signing identity is revoked, every component descended from it must be reachable and re-evaluated. The graph must record the publisher, build, and instance edges to make this a query.
- **Organizations & delegated administration.** An org is a principal that grants bounded administrative authority to sub-principals, without becoming an ambient super-uid (see [[multi_user_and_org_control]]).
- **Nested principals.** An app that holds authority over a synthetic realm ([[filesystem_as_database]]) mints its own sub-principals, its "users," with the same primitive the OS uses for real ones. An app's account model and the OS's principal model are one concept at two levels, and the app's minted principals are confined to its realm.
- **Remote attestation.** A principal across the network ([[distributed_boundary]]) must prove identity to the same standard as a local one. "Trusted because it's on the LAN" is exactly the ambient mistake to avoid.
- **Per-relationship identity.** Resisting correlation means presenting a distinct pseudonymous principal to each relying party by default, the way passkeys are per-site, so two services cannot link the same human across contexts. A stable cross-service identity is opt-in ([[wallet_and_credentials]]).
- **Federated login.** Authenticating against an organization or remote identity mints a session to the same standard as a local login. The OS brokers the credential, so the service receives a scoped authentication and never the user's secret ([[sessions_and_login]]).
- **Sessions.** A login session is a short-lived principal that carries a human's authority for a bounded window, and is itself revocable and leasable ([[capability_lifecycle]]).
- **Zero value.** A zeroed principal is the anonymous nobody: a valid, named graph node (shape 1 in [[omega_substrate]]) with unattested provenance that holds no authority. An uninitialized identity is the least-trusted holder rather than an error or, worse, an accidental match for a real principal.

## Key Questions

- What, concretely, is each identity kind, and what binds it across reinstall, rebuild, device move, and reboot?
- How is an identity rotated without invalidating the authority that legitimately flowed from it?
- When a publisher is compromised, what is the blast radius and how is it computed and contained?
- How are organizations and delegated administration modeled so they confer bounded authority rather than ambient power?

## Omega Leverage

- Principals are ordinary `data` values with domains expressing trust states (`Principal::Attested`, `Publisher::Revoked`); no new keyword.
- The authority-flow report already names who accepts, derives, and stores authority. Binding those holders to typed principals turns flow into a graph of named nodes.
- Ordinary numbered data under a selected codec is the natural carrier for an identity or attestation that crosses a boundary or persists across reboot.
- Omega does not define cross-principal attestation chains or identity rotation semantics. That trust-chain machinery is something Cathedral specifies on top ([[boot_and_trust_chain]]).

## Open Questions

- **The local-network-sharing facade.** Cross-world sharing is modeled as talking to a remote host. Is that literally loopback (`127.0.0.1` permitted) or a distinct local lowering? Does treating a sibling world as "remote" confuse the case where the user wants actual network sharing? Is targeted "share with exactly one other world" a credential concern, and do worlds identify themselves so one can whitelist a specific shared world? Does a shared world really need a full system realm? It is cheap (zero-copy hierarchy, storage decoupled), but is "a whole OS to share a file" the right shape?
- **Recovery without a backdoor.** A world sealed to a credential is unrecoverable if the credential is lost. Root must be able to reset a world without being able to read it, and a human wants to decouple from any single rooting device. That reset-not-read power is the hard residue ([[secrets_and_keys]], [[wallet_and_credentials]]).
- **Rotation.** How is a key or identity rotated without orphaning the authority that legitimately flowed from it? This is a graph-rewrite with no algorithm yet.

## Related
- [[capability_model]] — principals are the graph's nodes.
- [[secrets_and_keys]] — the keys that make an identity unforgeable.
- [[multi_user_and_org_control]] — organizations and delegated administration.
- [[audit_compliance_provenance]] — identity as a provenance record.
- [[store_and_economic_control]] — publisher identity in the distribution path.
- [[wallet_and_credentials]] — credentials and unlinkable per-relationship presentation.
