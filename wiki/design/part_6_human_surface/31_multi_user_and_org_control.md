# Chapter 31: Multi-User, Multi-Tenant & Org Control

> Whether a device serves one person, a family, a fleet, or a kiosk is a foundational decision — it shapes storage, identity, permissions, UI, and update policy, and it cannot be bolted on later.

## The Legacy Contract

Legacy multi-user grew out of the timesharing uid model and never escaped it. "User" conflates a human, a login account, a permission bundle, and a home directory. Enterprise management (MDM, group policy, domain join) is layered on top as a separate, often-fighting authority that the local OS only partially understands. Family devices, shared TVs, guest sessions, and work/personal separation are each handled by a different bespoke mechanism. Because tenancy was not a first-class isolation concept, "wipe only the work data," "this kiosk owns no personal state," and "the org may set this policy but not see that data" are all hard or impossible to state cleanly.

## What Cathedral Wants

Make the **tenant** a first-class isolation domain for *data, policy, and authority* (see the seed glossary in [[02_vocabulary]]), and decide the tenancy model *early* because it touches everything. A tenant scopes which data exists, which policies apply, and which capabilities are reachable; principals ([[05_identity_and_principals]]) act *within* a tenant. The same primitive must express the full range of deployments — and the design picks one model that covers them rather than a new mechanism per case:

- personal device (one tenant), family device (several), shared TV (profiles + guest), enterprise-managed device (org tenant with inherited policy), kiosk (ephemeral single-purpose tenant), server tenant (hard data/authority isolation), guest sessions, and work/personal separation on one device.

Org control is then *policy layered onto tenants* ([[20_configuration_and_policy]]) with clear inheritance, plus fleet-level actions (remote wipe, policy push) that respect tenant boundaries: the org may govern the work tenant without reaching into the personal one.

## Concerns & Design Space

- **Tenant as isolation domain.** Data, policy, and authority partitioned per tenant; nothing ambient crosses the boundary.
- **Profiles vs. tenants.** Lightweight profiles (family TV) vs. hard tenants (work/personal, server) — one model with strength dials, not two designs.
- **Policy inheritance & layering.** Org → device → tenant → app, with a defined resolution order and override rules ([[20_configuration_and_policy]]).
- **Data separation.** Per-tenant storage and privacy boundaries so "wipe the work tenant" is exact and complete ([[08_data_model_and_privacy]]).
- **Guest & ephemeral sessions.** Tenants that own no persistent state and leave no residue.
- **Remote wipe & managed deployment.** Fleet actions scoped to a tenant; managed enrollment via the store/control plane ([[36_store_and_economic_control]]).
- **Work/personal separation.** Two tenants on one device with a hard authority wall and a clear, spoof-resistant indicator of which one is active.
- **Fleet policy.** Tenancy as the unit a distributed control plane addresses ([[17_distributed_boundary]]).

## Key Questions

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
- [[05_identity_and_principals]] — principals acting within a tenant.
- [[08_data_model_and_privacy]] — per-tenant data separation.
- [[20_configuration_and_policy]] — policy layering and inheritance.
- [[17_distributed_boundary]] — tenancy as the unit of fleet policy.
- [[36_store_and_economic_control]] — managed deployment and enrollment.
