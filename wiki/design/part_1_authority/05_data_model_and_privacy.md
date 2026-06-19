# Chapter 05: Data Model & Privacy Boundaries

> Privacy is not a permission sheet stapled to a filesystem. This chapter owns data *classification* as structure — and access scoped by the *purpose* it serves.

## The Legacy Model

Legacy systems store everything as undifferentiated bytes in a flat namespace, and then bolt privacy on top as a per-app permission list: "this app may access Contacts," "this app may access Photos." The data has no inherent class — a JPEG of a medical record and a JPEG of a meme are the same object to the OS — so the permission is coarse, binary, and blind to *why* the access happens. Once an app is granted "Contacts," it may read every field of every contact for any reason, forever, and the system records only that it *was* granted, never *for what purpose*. Regulatory and audit regimes care about the *purpose* of access, not just whether it happened, and the legacy model cannot represent it.

## The Cathedral Model

Make classification **structural**. Every datum belongs to a **data class** the OS understands as a first-class thing — documents, media, contacts, messages, location, health, credentials, telemetry, app state, organization data, personal data, shared data — and authority over data is always *typed by its class*.

On top of that, the key idea: **purpose-scoped access**. A capability to read data carries the *purpose* it was granted for, and that purpose is checked and recorded.

```omega
Capability<Read<Contact.Email>, Purpose<SendMessage>>
Capability<Read<Location.Coarse>, Purpose<ShowNearbyStores>>
```

A grant of `Read<Contact.Email>` for `Purpose<SendMessage>` does not authorize the same read for `Purpose<BuildAdProfile>`. The purpose is not a comment; it is part of the capability, it bounds what the holder may do, and it lands in the provenance record ([[audit_compliance_provenance]]) so an auditor can answer "why was this contact's email read?" instead of merely "was Contacts granted?".

### The decided scope: the guarantee is confinement, not IFC

The privacy *guarantee* — "an app cannot exfiltrate your data" — is delivered by mechanisms already core, **not** by information-flow control:

- **Confinement / the capability boundary.** An app with no network capability *cannot* leak your data — there is no channel. "Don't let it exfiltrate" is mostly "don't hand it a channel," enforced at the boundary ([[capability_model]], [[security_policy_and_sandboxing]]).
- **Operation-capabilities for secrets.** Keys, credentials, wallet, and login are protected by *never holding the secret* — you hold a `Capability<Sign>` / `Capability<Unseal>`, and the value stays sealed / in hardware ([[secrets_and_keys]], [[wallet_and_credentials]]). Categorically stronger than labeling a held secret, and it does **not** use IFC.

So **information-flow control — propagating `Secret<T>`-style labels — is *not* the load-bearing privacy mechanism**, and is not needed to ship the OS, wallets, or login. It is the **fine-grained residual** for one case: an app that *legitimately holds* sensitive data **and** has an output channel, where some flows are allowed but the leak must be blocked (the photo app that may upload the picked photo but must not auto-exfiltrate the library). For that case the shape (from IFC prior art — Jif, HiStar, robust declassification) is: the label rides in the content-addressed envelope (stripping it = a *different* object); **declassification** (lowering a label) is a *gated, dangerous* op — unlike capability attenuation (lowering authority), which is *free* — and a declassify must also pin the **integrity of its input** or it is an authorized laundromat; **purpose** is the same machinery on the integrity axis.

**Layer + status:** IFC is a **language-level** feature (propagating labels need compiler support — not stdlib, not kernel-core), and it is **deferred / to-explore** — including whether a *less verbose* scheme than classical IFC exists (label creep is the historical adoption-killer; coarse labels + re-anchoring at world boundaries may suffice). The purpose-scoped capabilities and class-as-domain above remain the *recording/audit* surface; the *enforcement guarantee* is the capability boundary.

## Concerns & Design Space

- **Data classes as types, not folders.** Class is a property of the datum ([[filesystem_as_database]]), not of where it happens to sit; moving or copying a health record must not strip its class.
- **Purpose as a first-class scope.** Purposes form a vocabulary the OS and apps share; a capability binds class × operation × purpose. The hard part is keeping purposes meaningful rather than a checkbox an app self-asserts.
- **Field-level granularity.** `Contact.Email` is reachable without `Contact.*`; classification and access must reach into the shape of a datum, not just its type name.
- **Derived and shared data.** Data produced *from* classified data inherits class and purpose constraints; "shared data" and "organization data" are classes with their own propagation rules ([[multi_user_and_org_control]]).
- **Interaction with combination rules.** Data class is what [[security_policy_and_sandboxing]]'s dangerous-combination reasoning ranges over — `Read<Photo>` + `network_io` is dangerous *because* of the class.
- **Telemetry and credentials as classes.** Telemetry ([[telemetry_and_feedback]]) and credentials ([[secrets_and_keys]]) are data classes too; the OS vendor's own collection is bound by the same purpose-scoped model.
- **Minimization and expiry.** Purpose-scoping naturally supports "read for this task, then the grant lapses" — purpose plus leasing ([[capability_lifecycle]]).
- **Zero value.** A zeroed purpose is no declared purpose, which authorizes no use (shape 4 in [[omega_substrate]]): a purpose check against it fails safe, so unclassified or uninitialized data defaults to the most-restrictive class and an unpurposed read is denied rather than treated as universally permitted.

## Key Questions

- What is the canonical set of data classes, and is it closed (OS-defined) or extensible by apps and orgs?
- How is class *attached* to a datum and preserved across copy, transform, serialize, and IPC so it cannot be laundered away?
- What is a "purpose," who defines the vocabulary, and how is a holder prevented from using data acquired for one purpose to serve another?
- How does purpose flow through *derived* data — if app state is computed from location, does the result carry location's constraints?

## Omega Leverage

- **Data classes** map onto **domains** over `data` (`Datum::Health`, `Datum::Contact`) — named, provable predicates rather than runtime tags.
- **Purpose-scoped access** is a **typed capability** carrying both the class and the purpose; the authority-flow report then shows not just *what* data flows but *for what*.
- **`wire data`** preserves class/purpose annotations across boundaries.
- This is the chapter with the clearest **Omega gap**: capabilities are values + domains today, but *purpose-tagged authority* — a purpose that rides along a capability and constrains its use and propagation — is something Omega likely needs to grow for Cathedral to enforce it rather than merely record it.

## Open Questions

- Can purpose be *enforced* by the type system, or only *audited* after the fact? The privacy value differs enormously between the two.
- How is class assigned to data that arrives from outside (network, import, legacy box) with no inherent classification?
- Who adjudicates when an org's data-use policy conflicts with a user's privacy preference over the same datum ([[multi_user_and_org_control]])?

## Related
- [[capability_model]] — purpose-scoped reads are capabilities.
- [[security_policy_and_sandboxing]] — dangerous combinations range over classes.
- [[filesystem_as_database]] — where classified data actually lives.
- [[audit_compliance_provenance]] — purpose makes "why was this read?" answerable.
