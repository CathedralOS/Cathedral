# Chapter 04: Multi-User, Multi-Tenant & Org Control

> Whether a device serves one person, a family, a fleet, or a kiosk is a foundational decision — it shapes storage, identity, permissions, UI, and update policy, and it cannot be bolted on later.

## The Legacy Model

Legacy multi-user grew out of the timesharing uid model and never escaped it. "User" conflates a human, a login account, a permission bundle, and a home directory. Enterprise management (MDM, group policy, domain join) is layered on top as a separate, often-fighting authority that the local OS only partially understands. Family devices, shared TVs, guest sessions, and work/personal separation are each handled by a different bespoke mechanism. Because tenancy was not a first-class isolation concept, "wipe only the work data," "this kiosk owns no personal state," and "the org may set this policy but not see that data" are all hard or impossible to state cleanly.

## The Cathedral Model

Make the **tenant** a first-class isolation domain for *data, policy, and authority* (see the seed glossary in [[vocabulary]]), and decide the tenancy model *early* because it touches everything. A tenant scopes which data exists, which policies apply, and which capabilities are reachable; principals ([[identity_and_principals]]) act *within* a tenant. The same primitive must express the full range of deployments — and the design picks one model that covers them rather than a new mechanism per case:

- personal device (one tenant), family device (several), shared TV (profiles + guest), enterprise-managed device (org tenant with inherited policy), kiosk (ephemeral single-purpose tenant), server tenant (hard data/authority isolation), guest sessions, and work/personal separation on one device.

Org control is then *policy layered onto tenants* ([[configuration_and_policy]]) with clear inheritance, plus fleet-level actions (remote wipe, policy push) that respect tenant boundaries: the org may govern the work tenant without reaching into the personal one.

### The decided mechanism

**A "tenant" is a Matrix — not a new primitive.** A tenant is a confined world ([[identity_and_principals]]) plus its realms (data) and its ceiling (policy): the work tenant is a Matrix, the personal tenant a sibling Matrix behind a hard wall, a kiosk an ephemeral Matrix with a wipe-on-exit realm, a guest the *zero* Matrix, a server tenant an isolated Matrix. "One primitive or a family of kinds?" dissolves into **one primitive (the confined world) on a provisioning spectrum** — the strength dial is *how the Matrix is provisioned* (shared parent + soft boundary = a lightweight profile; isolated + no cross-exposure = a hard wall), the same single axis as the window→VM spectrum. *(Naming for "tenant"/"Matrix" is unsettled — placeholders.)*

**A "user" is a seated principal — not a primitive either.** "User" = a principal (the username/identity world) + the input devices routed to its seat ([[identity_and_principals]], [[windowing_and_compositor]]). So **multi-user and multi-agent are one model** — a seated principal — differing only in input source (physical/virtual, indistinguishable to consumers, OS-attested for the physical seat). A human acts through *per-persona* worlds (work / personal / anonymous, unlinkable), and crossing tenants is an **explicit, audited switch** with a spoof-resistant trusted-path indicator of which is active — never ambient spanning, which would *be* the cross-tenant leak.

**Isolation is structural, not a runtime tag.** A Matrix cannot name another's object because there is **no global root** and it holds no capability into the other's realm ([[filesystem_as_database]]) — cross-tenant naming is *unexpressible*, not detected-and-refused. Cross-tenant *sharing* is an explicit capability grant recorded in the graph, like any cross-realm share.

**Shared services isolate per-caller in *userspace*, not via OS machinery.** One printer or one network stack serving many tenants is a userspace provider each tenant reaches via its own capability; the *service* exposes a **per-caller view capability** (tenant A cannot see B's queue because the service attenuates per caller), and the OS only guarantees capabilities do not cross ambiently. So shared-service-without-leak is **service design + capability attenuation**, not OS machinery; the residual contention/timing channel is minor and the service's concern (the genuine *OS-level* shared-resource side-channel is the hardware-contention kind — `SteeringSlot` ([[networking]]) — not a print queue).

**Remote-wipe = crypto-erase the realm key + revoke the capability subtree.** "Wipe the work tenant" destroys the Matrix's sealed-realm key (the `wipe`/crypto-erase from [[filesystem_as_database]] — instant, no block-scrubbing) and revokes its capability subtree (generation bump, lazy-chain-walk [[capability_lifecycle]]). It is **exact and complete because the boundary was clean from the start** — no work data leaked into the personal realm. Shares resolve cleanly: a *copy* shared into personal is personal's (independent content-addressed cell, survives); a *live cap* into the work realm dies with it.

**Org control = a scoped ceiling-capability; policy is the ceiling-intersection.** org → device → tenant → app is the **most-restrictive-wins meet of ceilings** ([[security_policy_and_sandboxing]], [[configuration_and_policy]]) — everyone tightens, only the owner loosens. The org holds the **capability over the work-Matrix's ceiling**, scoped to *it*, which structurally cannot reach the personal Matrix (a sibling the org holds no capability over): "govern the work tenant without seeing the personal one," by construction.

**Orchestration is no-code for the common case.** Creating a Matrix is a capability (the realm-authority spawn-confined-child grant) held over your own world. The OS ships a **stock configurable default Matrix**, so the common case is right-click → New → pick a config preset (the world-chooser presets — Default / Locked-down / Throwaway / Trusted, [[human_permission_ux]]) → drag apps in (the cross-Matrix drag [[windowing_and_compositor]], re-minting them into the Matrix's realm). The child's `system:` binds to a **CoW overlay / view** of the real system realm ([[filesystem_as_database]] virtual realms) — *not a copy*, O(1), content-addressed-shared. Custom behavior means **overriding only the provider interface you care about** (compositor, network, realm view) and inheriting OS defaults for the rest — never reimplementing the whole world.

## Concerns & Design Space

- **Tenant as isolation domain.** Data, policy, and authority partitioned per tenant; nothing ambient crosses the boundary.
- **Profiles vs. tenants.** Lightweight profiles (family TV) vs. hard tenants (work/personal, server) — one model with strength dials, not two designs.
- **Policy inheritance & layering.** Org → device → tenant → app, with a defined resolution order and override rules ([[configuration_and_policy]]).
- **Data separation.** Per-tenant storage and privacy boundaries so "wipe the work tenant" is exact and complete ([[data_model_and_privacy]]).
- **Guest & ephemeral sessions.** Tenants that own no persistent state and leave no residue.
- **Remote wipe & managed deployment.** Fleet actions scoped to a tenant; managed enrollment via the store/control plane ([[store_and_economic_control]]).
- **Work/personal separation.** Two tenants on one device with a hard authority wall and a clear, spoof-resistant indicator of which one is active.
- **Fleet policy.** Tenancy as the unit a distributed control plane addresses ([[distributed_boundary]]).
- **Zero value.** A zero tenant is the valid-empty isolation domain ([[omega_substrate]] ZII): it owns no data, reaches no capability, and applies no policy, which is exactly the ephemeral guest or fresh-kiosk case, so acting within a zeroed tenant is coherent and leak-free rather than a cross-tenant fault.

## Key Questions

*(All four resolved by "The decided mechanism": one primitive — a confined-world Matrix — on a provisioning spectrum; policy inheritance = the most-restrictive-wins ceiling-intersection (only the owner loosens); remote-wipe = crypto-erase the realm key + revoke the capability subtree, exact because the boundary is clean; a human is a *seated principal* who crosses tenants only by an explicit audited switch, never ambient spanning.)*

- Is there one tenant primitive with strength parameters, or a small family of tenant kinds? What is the minimal model that covers personal → kiosk → server?
- How does policy inheritance resolve conflicts between org, device, and tenant layers, and who wins?
- What exactly does "remote wipe a tenant" guarantee about residual data and held capabilities?
- Where do humans sit relative to tenants — can one principal act across tenants, or is crossing always an explicit, audited switch?

## Omega Leverage

- A tenant boundary is expressible as a **domain / proof predicate** ([domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md)): capabilities and data carry their tenant; cross-tenant flow requires explicit, audited mediation.
- Tenant scoping rides the **capability** model — authority is reachable only if held *within* the tenant ([capabilities](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- Per-tenant state and wipe are **versioned data** with clean lifecycle ([versioned data](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)).
- Omega gives no built-in tenancy concept; the tenant-as-isolation-domain is a Cathedral construct layered on capabilities, domains, and storage.

## Open Questions

- Can tenant isolation be enforced statically (a capability simply cannot name another tenant's object), or does it need a runtime tenant tag checked at every boundary?
- How do shared system services (one printer, one network stack) serve multiple tenants without becoming a cross-tenant leak?

## Related
- [[identity_and_principals]] — principals acting within a tenant.
- [[data_model_and_privacy]] — per-tenant data separation.
- [[configuration_and_policy]] — policy layering and inheritance.
- [[distributed_boundary]] — tenancy as the unit of fleet policy.
- [[store_and_economic_control]] — managed deployment and enrollment.
