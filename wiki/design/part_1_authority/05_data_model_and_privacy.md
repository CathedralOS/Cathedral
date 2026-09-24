# Chapter 05: Data Model & Privacy Boundaries

> Privacy is not a permission sheet stapled to a filesystem. This chapter owns data classification as structure, and access scoped by the purpose it serves.

## The Legacy Model

A legacy system stores everything as undifferentiated bytes in a flat namespace and bolts privacy on top as a per-app permission list: "this app may access Contacts," "this app may access Photos." The data has no inherent class. A JPEG of a medical record and a JPEG of a meme are the same object to the OS, so the permission is coarse, binary, and blind to why the access happens. Once an app is granted "Contacts," it may read every field of every contact for any reason, forever, and the system records only that the grant was made, never what it was for. Regulatory and audit regimes care about the purpose of an access, not just whether it happened, and the legacy model cannot represent it.

## The Cathedral Model

Cathedral makes classification structural. Every datum belongs to a **data class** the OS understands as a thing in its own right (documents, media, contacts, messages, location, health, credentials, telemetry, app state, organization data, personal data, shared data), and authority over data is always typed by its class.

On top of that sits **purpose-scoped access**. A capability to read data carries the purpose it was granted for, and that purpose is checked and recorded.

```omega
Capability<Read<Contact.Email>, Purpose<SendMessage>>
Capability<Read<Location.Coarse>, Purpose<ShowNearbyStores>>
```

A grant of `Read<Contact.Email>` for `Purpose<SendMessage>` does not authorize the same read for `Purpose<BuildAdProfile>`. The purpose is part of the capability, not a comment. It bounds what the holder may do, and it lands in the provenance record ([[audit_compliance_provenance]]) so an auditor can answer "why was this contact's email read?" instead of only "was Contacts granted?".

### The decided scope: the guarantee is confinement, not IFC

The privacy guarantee, "an app cannot exfiltrate your data," is delivered by mechanisms that are already core, not by information-flow control:

- **Confinement at the capability boundary.** An app with no network capability cannot leak your data, because there is no channel. "Don't let it exfiltrate" is mostly "don't hand it a channel," enforced at the boundary ([[capability_model]], [[security_policy_and_sandboxing]]).
- **Operation-capabilities for secrets.** Keys, credentials, wallet, and login are protected by never holding the secret. You hold a `Capability<Sign>` or `Capability<Unseal>`, and the value stays sealed or in hardware ([[secrets_and_keys]], [[wallet_and_credentials]]). This is categorically stronger than labeling a held secret, and it does not use IFC.

Information-flow control, meaning the propagation of `Secret<T>`-style labels, is therefore not the load-bearing privacy mechanism, and it is not needed to ship the OS, wallets, or login. It is the fine-grained residual for one case: an app that legitimately holds sensitive data and also has an output channel, where some flows are allowed but the leak must be blocked. The photo app that may upload the picked photo but must not auto-exfiltrate the library is that case. Its shape comes from IFC prior art (Jif, HiStar, robust declassification). The label rides in the content-addressed envelope, so stripping it produces a different object. **Declassification**, lowering a label, is a gated and dangerous operation. Capability attenuation, lowering authority, is free by contrast. A declassify must also pin the integrity of its input, or it is an authorized laundromat. Purpose is the same machinery on the integrity axis.

IFC is a language-level feature, because propagating labels needs compiler support rather than stdlib or kernel-core support, and it is deferred and still to be explored. Part of that exploration is whether a less verbose scheme than classical IFC exists. Label creep is the historical adoption-killer, and coarse labels plus re-anchoring at world boundaries may suffice. The purpose-scoped capabilities and class-as-domain above are the recording and audit surface. The enforcement guarantee is the capability boundary.

## Concerns & Design Space

- **Data classes as types, not folders.** Class is a property of the datum ([[filesystem_as_database]]), not of where it happens to sit. Moving or copying a health record must not strip its class.
- **Purpose as a scope.** Purposes form a vocabulary the OS and apps share, and a capability binds class × operation × purpose. The hard part is keeping purposes meaningful rather than a checkbox an app self-asserts.
- **Field-level granularity.** `Contact.Email` is reachable without `Contact.*`. Classification and access must reach into the shape of a datum, not just its type name.
- **Derived and shared data.** Data produced from classified data inherits class and purpose constraints. "Shared data" and "organization data" are classes with their own propagation rules ([[multi_user_and_org_control]]).
- **Interaction with combination rules.** Data class is what the dangerous-combination reasoning in [[security_policy_and_sandboxing]] ranges over. `Read<Photo>` plus reach to the `Network` service is dangerous because of the class.
- **Telemetry and credentials as classes.** Telemetry ([[telemetry_and_feedback]]) and credentials ([[secrets_and_keys]]) are data classes too. The OS vendor's own collection is bound by the same purpose-scoped model.
- **Minimization and expiry.** Purpose-scoping supports "read for this task, then the grant lapses": purpose plus leasing ([[capability_lifecycle]]).
- **Zero value.** A zeroed purpose is no declared purpose, which authorizes no use (shape 4 in [[omega_substrate]]). A purpose check against it fails safe, so unclassified or uninitialized data defaults to the most restrictive class and an unpurposed read is denied rather than treated as universally permitted.

## Key Questions

- What is the canonical set of data classes, and is it closed (OS-defined) or extensible by apps and orgs?
- How is class attached to a datum and preserved across copy, transform, serialize, and IPC so it cannot be laundered away?
- What is a "purpose," who defines the vocabulary, and how is a holder prevented from using data acquired for one purpose to serve another?
- How does purpose flow through derived data? If app state is computed from location, does the result carry location's constraints?

## Omega Leverage

- Data classes map onto domains over `data` (`Datum::Health`, `Datum::Contact`): named, provable predicates rather than runtime tags.
- Purpose-scoped access is a typed capability carrying both the class and the purpose. The authority-flow report then shows not just what data flows but for what.
- Selected wire codecs over ordinary numbered schemas preserve class and purpose annotations across boundaries.
- This is the chapter with the clearest Omega gap. Omega's capabilities are values plus domains, but purpose-tagged authority, a purpose that rides along a capability and constrains its use and propagation, is something Omega likely needs to grow for Cathedral to enforce it rather than only record it.

## Open Questions

- Can purpose be enforced by the type system, or only audited after the fact? The privacy value differs enormously between the two.
- How is class assigned to data that arrives from outside (network, import, legacy box) with no inherent classification?
- Who adjudicates when an org's data-use policy conflicts with a user's privacy preference over the same datum ([[multi_user_and_org_control]])?

## Related
- [[capability_model]] — purpose-scoped reads are capabilities.
- [[security_policy_and_sandboxing]] — dangerous combinations range over classes.
- [[filesystem_as_database]] — where classified data actually lives.
- [[audit_compliance_provenance]] — purpose makes "why was this read?" answerable.
