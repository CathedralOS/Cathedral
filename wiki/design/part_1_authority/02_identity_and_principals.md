# Chapter 02: Identity, Principals & Trust

> Capabilities answer *what* may be done; this chapter owns *who* holds them — the principals that are the nodes of the authority graph, and why we trust them.

## The Legacy Model

Unix models identity as a small integer. A `uid` and a few `gid`s decide access, and authority is ambient over that identity: you can act because of *who you are*, not what you hold. The model has little vocabulary for the identities that carry the most weight; publisher identity and component provenance are not represented at the OS level. There is no first-class notion of an *app*, a *publisher*, a *component build*, a *device*, an *organization*, or a *session* — those are approximated with usernames, service accounts, signing certificates checked at install time, and MDM profiles layered on top. "Is this really the vendor who shipped the last update?" and "is this the same device that enrolled last week?" are answered outside the OS, if at all. Identity rotation means provisioning a new account and migrating files by hand.

## The Cathedral Model

A **principal** is any entity the authority graph can name as a holder of authority. Cathedral admits many kinds and treats them uniformly as graph nodes: human users, sessions, apps, individual components, package builds, publishers, devices, organizations, services, and remote principals across a trust boundary. Each principal has a stable, unforgeable identity with explicit *provenance* — we can say how the identity was established and who vouches for it — and that identity is what edges in the graph attach to (see [[capability_model]]).

The local human username is the *least* interesting of these. For a platform that distributes software, **publisher identity** and **component provenance** carry far more weight: the binding from a running component back to a signed build back to an attested publisher is the thing real trust decisions rest on.

```omega
data Principal {
    kind: PrincipalKind;     // User | App | Component | Publisher | Device | Org | Service | Session | Remote
    id: PrincipalId;         // stable, unforgeable
    provenance: Attestation; // who vouches, and how it was established
}

domain Principal::Attested { self.provenance in Attestation::Verified; }
```

## Concerns & Design Space

- **What an identity *is* per kind.** A user, an app, a publisher, a device, and a component are not the same shape. Each needs its own answer for what makes two references the same principal, and what survives reinstall, rebuild, or reboot.
- **Caller identity is kernel bookkeeping, not a claim.** A syscall is a trap by a thread the kernel itself scheduled, so "who is calling" is read from the task object — never from an argument, never forgeable, language-irrelevant. Running instances are named by **generational instance ids**, never recyclable pids (a stale reference to a dead instance is recognized-invalid, not a collision with its successor). The chain instance → measured image → manifest is forged at spawn, link by link, all kernel-maintained ([[boot_and_trust_chain]]) — which is what lets the grant arena check a manifest ceiling at delegation time ([[capability_lifecycle]]). The deliberate limit: identity does not propagate through intermediaries; when A asks service B to act, the kernel correctly sees B, and "on behalf of A" is a recorded delegation edge, not an impersonation claim — the confused deputy dies of explicitness.
- **Provenance over names.** Trust flows from attestation chains (publisher → build → component), not from a string. The chain is the identity.
- **Rotation.** Keys and identities must rotate without orphaning everything a principal held. This is a graph-rewrite problem, not a re-provisioning one, and it leans on key management ([[secrets_and_keys]]).
- **Compromised publishers.** When a publisher's signing identity is revoked, every component descended from it must be reachable and re-evaluated. The graph must record the publisher → build → instance edges to make this a query.
- **Organizations & delegated administration.** An org is a principal that grants bounded administrative authority to sub-principals — without becoming an ambient super-uid (see [[multi_user_and_org_control]]).
- **Nested principals.** An app that holds authority over a synthetic realm ([[filesystem_as_database]]) mints its own sub-principals, its "users," with the same primitive the OS uses for real ones. An app's account model and the OS's principal model are one concept at two levels, and the app's minted principals are confined to its realm.
- **Remote attestation.** A principal across the network ([[distributed_boundary]]) must prove identity to the same standard as a local one; "trusted because it's on the LAN" is exactly the ambient mistake to avoid.
- **Per-relationship identity.** Resisting correlation means presenting a distinct pseudonymous principal to each relying party by default, the way passkeys are per-site, so two services cannot link the same human across contexts; a stable cross-service identity is opt-in ([[wallet_and_credentials]]).
- **Federated login.** Authenticating against an organization or remote identity mints a session to the same standard as a local login, with the OS brokering the credential so the service receives a scoped authentication and never the user's secret ([[sessions_and_login]]).
- **Sessions.** A login/session is a short-lived principal that carries a human's authority for a bounded window, and is itself revocable and leasable ([[capability_lifecycle]]).
- **Zero value.** A zeroed principal is the anonymous nobody: a valid, named graph node (shape 1 in [[omega_substrate]]) with unattested provenance that holds no authority, so an uninitialized identity is the least-trusted holder rather than an error or, worse, an accidental match for a real principal.

## Key Questions

- What, concretely, *is* each identity kind, and what binds it across reinstall, rebuild, device move, and reboot?
- How is an identity rotated without invalidating the authority that legitimately flowed from it?
- When a publisher is compromised, what is the blast radius and how is it computed and contained?
- How are organizations and delegated administration modeled so they confer *bounded* authority rather than ambient power?

## Omega Leverage

- Principals are ordinary **`data`** values with **domains** expressing trust states (`Principal::Attested`, `Publisher::Revoked`); no new keyword.
- The **authority-flow report** already names who accepts/derives/stores authority; binding those holders to typed principals turns flow into a graph of named nodes.
- **`wire data`** with stable field numbers is the natural carrier for an identity or attestation that crosses a boundary or persists across reboot.
- Omega does **not** define cross-principal attestation chains or identity rotation semantics — that trust-chain machinery is something Cathedral specifies on top ([[boot_and_trust_chain]]).

## Open Questions

- Is there a single principal type with a kind tag, or a family of principal types with a shared trust interface?
- Where does the root of identity live — a hardware device identity, an enrollment authority, the store — and who is allowed to mint a new principal?
- Can a human's identity be fully decoupled from any one device, and what is the recovery story when the rooting device is lost?

## Related
- [[capability_model]] — principals are the graph's nodes.
- [[secrets_and_keys]] — the keys that make an identity unforgeable.
- [[multi_user_and_org_control]] — organizations and delegated administration.
- [[audit_compliance_provenance]] — identity as a provenance record.
- [[store_and_economic_control]] — publisher identity in the distribution path.
- [[wallet_and_credentials]] — credentials and unlinkable per-relationship presentation.
