# Chapter 03: Naming & Discovery

> A name resolves to authority, and a spoofed name steals it — so Cathedral resolves to authority by **unforgeable value (a capability or a content-hash/key), not by string**. Names barely exist: the internals are nameless, local names are mundane, and the one hard case — a human-memorable global name for a stranger — is the parked network trust-bootstrap.

## The Legacy Model

Naming in legacy systems is a dozen uncoordinated schemes that calcify into permanent compatibility constraints — filesystem paths, DNS names, D-Bus service names, device nodes, package names, usernames, MIME types, URL schemes, port numbers — each invented separately, each with different uniqueness, ownership, and spoofing properties. None was designed as a security boundary, yet all of them resolve to authority: a path resolves to a file you may write, a service name to a process you will trust, a package name to code you will run. Homograph domains, dependency-confusion package names, and service-name hijacking are all the same bug — *a name resolved to the wrong authority*. And strings sit everywhere, resolved constantly at real compute cost, even where nothing human ever reads them.

## The Cathedral Model

Designation is by **unforgeable value, not string**: a **capability** (designation-by-reference — you hold it, there is no name) or a **content-hash / key** (a stable, unforgeable identity). So the threat "what authority does an attacker gain by making me resolve their name instead of the intended one" mostly **has no attack surface, because there is no string to resolve** — you hold a capability, or you name a hash. Human-readable strings appear only where a *human* must read, type, or share something, and there they are **aliases layered over the unforgeable identity, never the identity itself**.

The consequence is that "naming" is not a subsystem. It splits into three, and only the third is hard.

### 1. The internals are nameless

Capabilities designate by reference ([[capability_model]]) — you reach a service, object, or principal because you *hold a capability to it*, not because you resolved a string. Where a persistent identity is needed across reboot or machine, it is a **content-hash or key**: an app is its closure-hash / signing-key, an object its content-address, a protocol its schema-hash, a principal its realm-key. The `com.vendor.app`, `service://…`, and `protocol://…/v3` strings people write are **store/dev display labels over those unforgeable values**, not names the OS resolves to authority — you never "resolve Spotify" at runtime, you *hold the closure*. So the entire machine fabric has **no names to spoof, and none to resolve at cost**.

### 2. Local names are mundane, scoped, and owned

Human names *do* appear locally — a realm path is a chain of filenames, an environment binds "default printer" or "default camera." But these are **local labels in a scoped, owned namespace**: a realm path resolves to an object capability within *your* realm; an environment binding resolves within *your* per-principal resolution environment (the recursive-provider, [[capability_model]]). There is **no global-uniqueness question and no spoofing surface** — it is yours. The only mechanics are boring: a separator convention (disallowed inside a name), and per-principal resolution so a host can decide what a name means for a child (split-horizon, fabricated or blocked entries) — both already decided. A path is a human-convenience traversal whose leaves are object capabilities; the names are labels on the edges, never the identity.

### 3. Human-memorable global names — the one hard case, and it is the parked network problem

The single place a name must resolve to authority you do *not* already hold, by something a human can type / remember / share, is **first contact with a stranger**: a URL, a person's handle, a remote service. This — and only this — is naming-as-a-security-primitive (homograph, squatting, hijack), because here the string genuinely stands in for authority you have no prior reference to. And it is **not an OS naming system; it is the network trust-bootstrap** — the **first-pin** problem, parked in the security bucket and explored in [[networking]] / [post_dns_resolution](../../speculation/post_dns_resolution.md). The self-certifying half is clear: a name that *is* a key resolves by authenticating end-to-end to that key, no third party. The hard, parked half is binding a **human-memorable** string to a key for a **cold stranger**. The theoretical direction floated for the "reference a known remote endpoint" case is an **untrusted distributed hash table of public keys** — recorded as a possible approach, not a near-term commitment.

## The through-line

Everywhere the OS resolves to authority it uses an **unforgeable value** — a capability or a content-hash/key — so there is **nothing to spoof and nothing to resolve at cost**. Strings appear only where a human must read them, and only as **aliases over the unforgeable identity**: local labels (mundane, scoped, owned) or first-contact handles (the parked network trust-bootstrap). Avoiding strings anywhere they are not human-facing is deliberate — they add compute and a spoofing surface for no benefit.

## Concerns & Design Space

- **Unforgeable identity underneath.** A typed family of unforgeable values per kind — app = key/closure-hash, object = content-address, service = endpoint capability, protocol = schema-hash, principal = realm-key. One property, not one scheme: the identity is always an unforgeable value.
- **Human names as aliases over stable IDs.** Display names are aliases of the unforgeable identity; identity survives renaming because identity *is* the stable ID, not the string.
- **Per-principal resolution.** A local name's binding is resolved in each principal's environment, so a host decides what names mean for a child — the recursive-provider applied to naming, shared with realms ([[filesystem_as_database]]) and nested networking ([[networking]]).
- **Spoof resistance is structural.** Homograph, confusable, squatting, and dependency-confusion attacks die by construction, because authority binds to the unforgeable ID and the string was never the authority.
- **Renaming, aliases, deprecation.** A rename rebinds a local alias to the same stable ID; the stable ID (key/hash) is **never reused**, so a freed name cannot inherit authority; a local alias is rebindable only by its owner and pinned (trust-on-first-use), so a rebind to a different key is a *detected change*, not a silent inheritance.
- **No global namespace authority.** The OS arbitrates no global namespace; the only global-name case is first-contact, handled as federated self-certifying (a name that is a key), never a central resolver.
- **Zero value.** A zero name resolves to the canonical null object ([[omega_substrate]] ZII): resolution hands back the zero capability over nothing, valid-empty rather than a fault, and never silently resolves to some other principal's authority.

## Key Questions

- **The stable identifier — resolved:** a typed family of unforgeable values (key / content-hash / capability) per kind; the human name is a display alias over it, never the identity.
- **What proves a claimant owns a name — resolved:** *local* names resolve through the host's per-principal environment (the host *is* the authority for its children's namespace, honor-the-sandbox); *global self-certifying* names resolve by authenticating end-to-end to the key the name *is* (no third party); *human-memorable global name → key* is **first-pin, parked**. Resolution returns a capability attenuated to exactly that authority.
- **Aliases and renames — resolved:** local rebindings over the stable ID; because identity is the ID, a rename changes nothing the authority graph tracks.
- **Name reuse — resolved:** the stable ID is never reused (structurally kills authority-inheritance); a local alias is rebindable only by its owner and pinned to detect key-changes.

## Omega Leverage

- Names and IDs are **values**; resolution yields a **capability + domain**, so a resolved name carries exactly its authority and no more ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Self-certifying spoof-resistance lives in a **domain / proof predicate** ([domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md)): a name is `Resolvable::Authentic` only with proof it authenticates to its key.
- Durable names ride on explicit historical schemas and migrations; names
  crossing boundaries use numbered protocol data and explicit codecs.
- Omega defines no global namespace authority; who arbitrates first-contact uniqueness is the parked network/security question, not a language feature.

## Open Questions

- **Human-memorable global name → cold-stranger key (first-pin)** — the only genuinely-hard residual, parked in the security bucket. The self-certifying (name-is-a-key) part is decided; the human-memorable-binding part needs the security expert. The untrusted-DHT-of-public-keys is the recorded speculative approach, not a near-term commitment ([post_dns_resolution](../../speculation/post_dns_resolution.md)).

## Related
- [[capability_model]] — designation by reference; per-principal resolution.
- [[filesystem_as_database]] — realm paths as local label chains over object capabilities.
- [[identity_and_principals]] — app/principal identity as keys.
- [[ipc_and_service_invocation]] — services reached by capability, discovered via a scoped broker, not a global name.
- [[networking]] — first-contact and the network trust-bootstrap.
