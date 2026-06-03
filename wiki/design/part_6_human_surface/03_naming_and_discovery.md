# Chapter 03: Naming & Discovery

> Every OS grows naming systems — and naming is a security primitive, because a name resolves to authority, and a spoofed name steals it.

## The Legacy Model

Naming in legacy systems is a dozen uncoordinated schemes that calcify into permanent compatibility constraints. Filesystem paths, DNS names, D-Bus service names, device nodes, package names, usernames, MIME types, URL schemes, port numbers — each invented separately, each with different uniqueness, ownership, and spoofing properties. None was designed as a security boundary, yet all of them resolve to authority: a path resolves to a file you may write, a service name to a process you will trust, a package name to code you will run. Homograph domains, dependency-confusion package names, and service-name hijacking are all the same bug — *a name resolved to the wrong authority*. And once a naming scheme ships, its mistakes are forever; renaming breaks the world.

## The Cathedral Model

One coherent naming discipline where **a name is a value that resolves to authority through an auditable path**, and where resolution is *spoof-resistant by construction*. Names are typed, namespaced, and owned: resolving a name yields a capability (or a principal handle), and the resolution records *who* claimed that name and *whether that claim is trustworthy*. Human-readable names are layered *over* stable, unforgeable identifiers, never the other way around, so that renaming and aliasing are first-class operations that do not change identity.

Worked shapes the design must support cleanly:

- `com.vendor.app` — app identity ([[identity_and_principals]]).
- `service://camera.default` — a service/protocol name ([[ipc_and_service_invocation]]).
- `object://user/photos/2026/...` — a stable object ID ([[filesystem_as_database]]).
- `protocol://vendor.payment/v3` — a versioned protocol name.

**Key principle: naming is a security primitive.** The threat model for every name is "what authority does an attacker gain by getting me to resolve their name instead of the intended one?"

## Concerns & Design Space

- **Global uniqueness** vs. local convenience — when must a name be globally unique, and who arbitrates the global namespace?
- **Human-readable names over stable IDs.** Display names are aliases of unforgeable identifiers; identity survives renaming.
- **Renaming & aliases.** First-class, non-identity-changing, recorded — multiple names may map to one object/principal.
- **Versioning.** Names carry or resolve a version (`/v3`); resolution honors compatibility ([[versioned_state_and_migration]]).
- **Discovery.** Finding services/objects/peers by name or attribute, with resolution that returns *attenuated* authority, not the whole namespace.
- **Spoof resistance.** Homograph, confusable, and squatting attacks neutralized at the resolution layer; the resolver authenticates claims to a name.
- **Ownership transfer.** Handing a name to a new owner without orphaning what it pointed to or silently redirecting trust.
- **Deprecation.** Retiring a name without breaking holders, and without an attacker reclaiming a freed name to inherit its authority.

## Key Questions

- What is the stable, unforgeable identifier underneath each human name, and is it one scheme or a small typed family per kind (app / service / object / protocol)?
- At resolution time, what proves a name's claimant is its legitimate owner — and what capability does resolution hand back?
- How are aliases and renames recorded so the authority graph stays honest about "this name now means that principal"?
- Can a deprecated/freed name ever be safely reused, or is reuse forbidden to prevent authority inheritance?

## Omega Leverage

- Names and IDs are **values**; resolution yields a **capability + domain**, so a resolved name carries exactly its authority and no more ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- Spoof-resistance lives in a **domain / proof predicate** ([domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md)): a name is in `Resolvable::Authentic` only with proof of its owner's claim.
- Versioned names ride on **versioned data** ([versioned data](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)).
- Wire-format names crossing boundaries are **wire data** ([wire protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md)).
- Omega does not define a global namespace authority; who arbitrates uniqueness is a Cathedral governance question, not a language feature.

## Open Questions

- Is there one universal name resolver, or a federation per namespace with shared spoof-resistance rules?
- How much of resolution can be proved statically versus checked at runtime against a live registry?

## Related
- [[identity_and_principals]] — app IDs and principal names.
- [[capability_model]] — resolution yields authority.
- [[ipc_and_service_invocation]] — service and protocol names.
- [[filesystem_as_database]] — stable object IDs.
- [[networking]] — network names and their spoofing surface.
