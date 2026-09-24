# Chapter 03: Naming & Discovery

> A name resolves to authority, and a spoofed name steals it. Cathedral resolves to authority by unforgeable value, a capability or a content-hash or key, not by string. Names barely exist: the internals are nameless, local names are mundane, and the one hard case, a human-memorable global name for a stranger, is the network trust-bootstrap and belongs to the security work.

## The Legacy Model

Naming in legacy systems is a dozen uncoordinated schemes that calcify into permanent compatibility constraints: filesystem paths, DNS names, D-Bus service names, device nodes, package names, usernames, MIME types, URL schemes, port numbers. Each was invented separately, and each has different uniqueness, ownership, and spoofing properties. None was designed as a security boundary, yet all of them resolve to authority. A path resolves to a file you may write, a service name to a process you will trust, a package name to code you will run. Homograph domains, dependency-confusion package names, and service-name hijacking are all the same bug: a name resolved to the wrong authority. And strings sit everywhere, resolved constantly at real compute cost, even where nothing human ever reads them.

## The Cathedral Model

Designation is by unforgeable value, not string. The value is either a **capability**, which designates by reference (you hold it, and there is no name), or a **content-hash or key**, which is a stable, unforgeable identity. The threat "what authority does an attacker gain by making me resolve their name instead of the intended one" mostly has no attack surface, because there is no string to resolve. You hold a capability, or you name a hash. Human-readable strings appear only where a human must read, type, or share something, and there they are aliases layered over the unforgeable identity, never the identity itself.

The consequence is that naming is not a subsystem. It splits into three parts, and only the third is hard.

### 1. The internals are nameless

Capabilities designate by reference ([[capability_model]]). You reach a service, object, or principal because you hold a capability to it, not because you resolved a string. Where a persistent identity is needed across reboot or machine, it is a content-hash or key: an app is its closure-hash or signing key, an object its content-address, a protocol its schema-hash, a principal its realm-key. The `com.vendor.app`, `service://…`, and `protocol://…/v3` strings people write are store and developer display labels over those unforgeable values, not names the OS resolves to authority. You never "resolve Spotify" at runtime. You hold the closure. The entire machine fabric has no names to spoof and none to resolve at cost.

### 2. Local names are mundane, scoped, and owned

Human names do appear locally. A realm path is a chain of filenames, and an environment binds "default printer" or "default camera". These are local labels in a scoped, owned namespace. A realm path resolves to an object capability within your realm. An environment binding resolves within your per-principal resolution environment, which is the recursive provider of [[capability_model]]. There is no global-uniqueness question and no spoofing surface, because the namespace is yours. The only mechanics are boring: a separator convention, disallowed inside a name, and per-principal resolution so a host can decide what a name means for a child through split-horizon, fabricated, or blocked entries. The host is the authority for its children's namespace, which is the honor-the-sandbox rule applied to names. A path is a human-convenience traversal whose leaves are object capabilities. The names are labels on the edges, never the identity.

### 3. Human-memorable global names are the one hard case, and they are the network problem

The single place a name must resolve to authority you do not already hold, by something a human can type, remember, or share, is first contact with a stranger: a URL, a person's handle, a remote service. This, and only this, is naming as a security primitive (homograph, squatting, hijack), because here the string stands in for authority you have no prior reference to. It is not an OS naming system. It is the network trust-bootstrap, the **first-pin** problem, which sits in the security bucket and is explored in [[networking]] and [post_dns_resolution](../../speculation/post_dns_resolution.md). The self-certifying half is clear: a name that is a key resolves by authenticating end-to-end to that key, with no third party. The hard half, still open, is binding a human-memorable string to a key for a cold stranger. The theoretical direction floated for the "reference a known remote endpoint" case is an untrusted distributed hash table of public keys, recorded as a possible approach, not a near-term commitment.

## The through-line

Everywhere the OS resolves to authority it uses an unforgeable value, a capability or a content-hash or key, so there is nothing to spoof and nothing to resolve at cost. Strings appear only where a human must read them, and only as aliases over the unforgeable identity: local labels, which are mundane, scoped, and owned, or first-contact handles, which are the network trust-bootstrap. Strings are avoided anywhere they are not human-facing because they add compute and a spoofing surface for no benefit.

## Concerns & Design Space

- **Unforgeable identity underneath.** A typed family of unforgeable values, one per kind. An app is a key or closure-hash, an object a content-address, a service an endpoint capability, a protocol a schema-hash, a principal a realm-key. This is one property, not one scheme: the identity is always an unforgeable value.
- **Human names as aliases over stable IDs.** Display names are aliases of the unforgeable identity. Identity survives renaming because identity is the stable ID, not the string, so a rename changes nothing the authority graph tracks.
- **Per-principal resolution.** A local name's binding is resolved in each principal's environment, so a host decides what names mean for a child. This is the recursive provider applied to naming, shared with realms ([[filesystem_as_database]]) and nested networking ([[networking]]).
- **Spoof resistance is structural.** Homograph, confusable, squatting, and dependency-confusion attacks die by construction, because authority binds to the unforgeable ID and the string was never the authority.
- **Renaming, aliases, deprecation.** A rename rebinds a local alias to the same stable ID. The stable ID (key or hash) is never reused, so a freed name cannot inherit authority. A local alias is rebindable only by its owner and is pinned (trust on first use), so a rebind to a different key is a detected change, not a silent inheritance.
- **No global namespace authority.** The OS arbitrates no global namespace. The only global-name case is first contact, handled as federated self-certifying naming (a name that is a key), never a central resolver.
- **Zero value.** A zero name resolves to the canonical null object ([[omega_substrate]] ZII). Resolution hands back the zero capability over nothing, valid-empty rather than a fault, and never silently resolves to some other principal's authority.

## Key Questions

- What is the stable identifier beneath a human name, for each kind of thing named? The chapter's answer is a typed family of unforgeable values (key, content-hash, capability), with the human name as a display alias over it.
- What proves a claimant owns a name, locally and globally, and what does resolution hand back? Local names resolve through the host's per-principal environment, self-certifying global names by authenticating to the key the name is, and either way resolution returns a capability attenuated to exactly that authority.
- How do aliases and renames interact with the authority graph?
- What stops a reused or rebound name from inheriting authority?

## Omega Leverage

- Names and IDs are values, and resolution yields a capability plus domain, so a resolved name carries exactly its authority and no more ([capabilities chapter](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Self-certifying spoof resistance lives in a domain or proof predicate ([domains](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_8_domains.md)): a name is `Resolvable::Authentic` only with proof it authenticates to its key.
- Versioned names ride on historical ordinary data. Names crossing boundaries use the selected numbered schema and wire codec.
- Omega defines no global namespace authority. Who arbitrates first-contact uniqueness is the network and security question, not a language feature.

## Open Questions

- How does a human-memorable global name bind to a cold stranger's key? This is the first-pin problem, the one hard residual, and it sits in the security bucket. The self-certifying part, a name that is a key, is designed. The human-memorable binding needs the security expert. The untrusted distributed hash table of public keys is the recorded speculative approach, not a near-term commitment ([post_dns_resolution](../../speculation/post_dns_resolution.md)).

## Related
- [[capability_model]] — designation by reference; per-principal resolution.
- [[filesystem_as_database]] — realm paths as local label chains over object capabilities.
- [[identity_and_principals]] — app/principal identity as keys.
- [[ipc_and_service_invocation]] — services reached by capability, discovered via a scoped broker, not a global name.
- [[networking]] — first-contact and the network trust-bootstrap.
