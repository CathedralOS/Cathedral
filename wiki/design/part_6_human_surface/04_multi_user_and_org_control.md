# Chapter 04: Multi-User, Multi-Tenant & Org Control

> Whether a device serves one person, a family, a fleet, or a kiosk is a foundational decision. It shapes storage, identity, permissions, UI, and update policy, and it cannot be bolted on later.

## The Legacy Model

Legacy multi-user grew out of the timesharing uid model and never escaped it. "User" conflates a human, a login account, a permission bundle, and a home directory. Enterprise management (MDM, group policy, domain join) is layered on top as a separate, often-fighting authority that the local OS only partially understands. Family devices, shared TVs, guest sessions, and work/personal separation are each handled by a different bespoke mechanism. Because tenancy was never an isolation concept in its own right, "wipe only the work data", "this kiosk owns no personal state", and "the org may set this policy but not see that data" are all hard or impossible to state cleanly.

## The Cathedral Model

The **tenant** is an isolation domain for data, policy, and authority (see the seed glossary in [[vocabulary]]), and the tenancy model is decided early because it touches everything. A tenant scopes which data exists, which policies apply, and which capabilities are reachable. Principals ([[identity_and_principals]]) act within a tenant. One primitive expresses the full range of deployments, rather than a new mechanism per case:

- personal device (one tenant), family device (several), shared TV (profiles plus guest), enterprise-managed device (org tenant with inherited policy), kiosk (ephemeral single-purpose tenant), server tenant (hard data and authority isolation), guest sessions, and work/personal separation on one device.

Org control is then policy layered onto tenants ([[configuration_and_policy]]) with clear inheritance, plus fleet-level actions (remote wipe, policy push) that respect tenant boundaries. The org may govern the work tenant without reaching into the personal one.

### The decided mechanism

A tenant is a Matrix, not a new primitive. A tenant is a confined world ([[identity_and_principals]]) plus its realms (data) and its ceiling (policy). The work tenant is a Matrix, the personal tenant a sibling Matrix behind a hard wall, a kiosk an ephemeral Matrix with a wipe-on-exit realm, a guest the zero Matrix, a server tenant an isolated Matrix. The question "one primitive or a family of kinds?" dissolves into one primitive, the confined world, on a provisioning spectrum. The strength dial is how the Matrix is provisioned: a shared parent with a soft boundary is a lightweight profile, and an isolated Matrix with no cross-exposure is a hard wall. This is the same single axis as the window-to-VM spectrum.

A user is a seated principal, and not a primitive either. A user is a principal (the username or identity world) plus the input devices routed to its seat ([[identity_and_principals]], [[windowing_and_compositor]]). Multi-user and multi-agent are therefore one model, the seated principal, differing only in input source. Physical and virtual input are indistinguishable to consumers, and the physical seat is OS-attested. A human acts through per-persona worlds (work, personal, anonymous, unlinkable). Crossing tenants is an audited switch with a spoof-resistant trusted-path indicator of which is active. It is never ambient spanning, because ambient spanning would be the cross-tenant leak.

Isolation is structural, not a runtime tag. A Matrix cannot name another's object because there is no global root and it holds no capability into the other's realm ([[filesystem_as_database]]). Cross-tenant naming is unexpressible, not detected and refused. Cross-tenant sharing is a capability grant recorded in the graph, like any cross-realm share.

Shared services isolate per caller in userspace, not through OS machinery. One printer or one network stack serving many tenants is a userspace provider that each tenant reaches through its own capability. The service exposes a **per-caller view capability**: tenant A cannot see B's queue because the service attenuates per caller, and the OS guarantees only that capabilities do not cross ambiently. Shared-service-without-leak is service design plus capability attenuation. The residual contention and timing channel is minor and is the service's concern. The OS-level shared-resource side-channel is the hardware-contention kind, `SteeringSlot` in [[networking]], not a print queue.

Remote wipe is crypto-erase of the realm key plus revocation of the capability subtree. "Wipe the work tenant" destroys the Matrix's sealed-realm key (the `wipe` crypto-erase from [[filesystem_as_database]], which is instant, with no block scrubbing) and revokes its capability subtree (a generation bump and lazy chain-walk, [[capability_lifecycle]]). It is exact and complete because the boundary was clean from the start: no work data leaked into the personal realm. Shares resolve cleanly. A copy shared into personal belongs to personal, as an independent content-addressed cell, and survives. A live capability into the work realm dies with it.

Org control is a scoped ceiling capability, and policy is the ceiling intersection. The chain org → device → tenant → app is a most-restrictive-wins meet of ceilings ([[security_policy_and_sandboxing]], [[configuration_and_policy]]). Everyone tightens, and only the owner loosens. The org holds the capability over the work Matrix's ceiling, scoped to that Matrix, which structurally cannot reach the personal Matrix, a sibling the org holds no capability over. "Govern the work tenant without seeing the personal one" holds by construction.

Orchestration is no-code for the common case. Creating a Matrix is a capability (the realm-authority spawn-confined-child grant) held over your own world. The OS ships a stock configurable default Matrix, so the common case is right-click, New, pick a config preset (the world-chooser presets Default, Locked-down, Throwaway, and Trusted, [[human_permission_ux]]), and drag apps in (the cross-Matrix drag of [[windowing_and_compositor]], which re-mints them into the Matrix's realm). The child's `system:` binds to a CoW overlay or view of the real system realm ([[filesystem_as_database]] virtual realms), not a copy: O(1) and content-addressed-shared. Custom behavior means overriding only the provider interface you care about (compositor, network, realm view) and inheriting OS defaults for the rest, never reimplementing the whole world.

## Concerns & Design Space

- **Tenant as isolation domain.** Data, policy, and authority are partitioned per tenant. Nothing ambient crosses the boundary.
- **Profiles vs. tenants.** Lightweight profiles (family TV) and hard tenants (work/personal, server) are one model with strength dials, not two designs.
- **Policy inheritance & layering.** Org → device → tenant → app, with a defined resolution order and override rules ([[configuration_and_policy]]).
- **Data separation.** Per-tenant storage and privacy boundaries so "wipe the work tenant" is exact and complete ([[data_model_and_privacy]]).
- **Guest & ephemeral sessions.** Tenants that own no persistent state and leave no residue.
- **Remote wipe & managed deployment.** Fleet actions scoped to a tenant, and managed enrollment through the store or control plane ([[store_and_economic_control]]).
- **Work/personal separation.** Two tenants on one device with a hard authority wall and a clear, spoof-resistant indicator of which one is active.
- **Fleet policy.** Tenancy is the unit a distributed control plane addresses ([[distributed_boundary]]).
- **Zero value.** A zero tenant is the valid-empty isolation domain ([[omega_substrate]] ZII). It owns no data, reaches no capability, and applies no policy, which is exactly the ephemeral guest or fresh-kiosk case, so acting within a zeroed tenant is coherent and leak-free rather than a cross-tenant fault.

## Key Questions

- Is there one tenant primitive with strength parameters, or a small family of tenant kinds? What is the minimal model that covers personal, kiosk, and server?
- How does policy inheritance resolve conflicts between org, device, and tenant layers, and who wins?
- What exactly does "remote wipe a tenant" guarantee about residual data and held capabilities?
- Where do humans sit relative to tenants? Can one principal act across tenants, or is crossing always an audited switch?
- Can tenant isolation be enforced statically, so that a capability cannot name another tenant's object, or does it need a runtime tenant tag checked at every boundary?
- How do shared system services (one printer, one network stack) serve multiple tenants without becoming a cross-tenant leak?

## Omega Leverage

- A tenant boundary is expressible as a domain or proof predicate ([domains](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_8_domains.md)): capabilities and data carry their tenant, and cross-tenant flow requires audited mediation.
- Tenant scoping rides the capability model. Authority is reachable only if held within the tenant ([capabilities](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Per-tenant state uses immutable historical schemas and checked conversions with a clean lifecycle ([historical data](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#versioned-data)).
- Omega gives no built-in tenancy concept. The tenant as isolation domain is a Cathedral construct layered on capabilities, domains, and storage.

## Open Questions

- What are the final names for "tenant" and "Matrix"? Both are placeholders.

## Related
- [[identity_and_principals]] — principals acting within a tenant.
- [[data_model_and_privacy]] — per-tenant data separation.
- [[configuration_and_policy]] — policy layering and inheritance.
- [[distributed_boundary]] — tenancy as the unit of fleet policy.
- [[store_and_economic_control]] — managed deployment and enrollment.
