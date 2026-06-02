# Chapter 08: Data Model & Privacy Boundaries

> Privacy is not a permission sheet stapled to a filesystem. This chapter owns
> data *classification* as structure — and access scoped by the *purpose* it serves.

## The Legacy Contract

Legacy systems store everything as undifferentiated bytes in a flat namespace,
and then bolt privacy on top as a per-app permission list: "this app may access
Contacts," "this app may access Photos." The data has no inherent class — a JPEG
of a medical record and a JPEG of a meme are the same object to the OS — so the
permission is coarse, binary, and blind to *why* the access happens. Once an app
is granted "Contacts," it may read every field of every contact for any reason,
forever, and the system records only that it *was* granted, never *for what
purpose*. Enterprise and regulatory regimes (the ones that actually audit data
use) care precisely about the *why*, and the legacy model cannot represent it.

## What Cathedral Wants

Make classification **structural**. Every datum belongs to a **data class** the
OS understands as a first-class thing — documents, media, contacts, messages,
location, health, credentials, telemetry, app state, organization data, personal
data, shared data — and authority over data is always *typed by its class*.

On top of that, the key idea: **purpose-scoped access**. A capability to read
data carries the *purpose* it was granted for, and that purpose is checked and
recorded.

```omega
Capability<Read<Contact.Email>, Purpose<SendMessage>>
Capability<Read<Location.Coarse>, Purpose<ShowNearbyStores>>
```

A grant of `Read<Contact.Email>` for `Purpose<SendMessage>` does not authorize
the same read for `Purpose<BuildAdProfile>`. The purpose is not a comment; it is
part of the capability, it bounds what the holder may do, and it lands in the
provenance record ([[34_audit_compliance_provenance]]) so an auditor can answer
"why was this contact's email read?" instead of merely "was Contacts granted?".

## Concerns & Design Space

- **Data classes as types, not folders.** Class is a property of the datum
  ([[18_filesystem_as_database]]), not of where it happens to sit; moving or
  copying a health record must not strip its class.
- **Purpose as a first-class scope.** Purposes form a vocabulary the OS and apps
  share; a capability binds class × operation × purpose. The hard part is keeping
  purposes meaningful rather than a checkbox an app self-asserts.
- **Field-level granularity.** `Contact.Email` is reachable without `Contact.*`;
  classification and access must reach into the shape of a datum, not just its
  type name.
- **Derived and shared data.** Data produced *from* classified data inherits class
  and purpose constraints; "shared data" and "organization data" are classes with
  their own propagation rules ([[31_multi_user_and_org_control]]).
- **Interaction with combination rules.** Data class is what
  [[06_security_policy_and_sandboxing]]'s dangerous-combination reasoning ranges
  over — `Read<Photo>` + `network_io` is dangerous *because* of the class.
- **Telemetry and credentials as classes.** Telemetry ([[35_telemetry_and_feedback]])
  and credentials ([[07_secrets_and_keys]]) are data classes too; the OS vendor's
  own collection is bound by the same purpose-scoped model.
- **Minimization and expiry.** Purpose-scoping naturally supports "read for this
  task, then the grant lapses" — purpose plus leasing ([[04_capability_lifecycle]]).

## Key Questions

- What is the canonical set of data classes, and is it closed (OS-defined) or
  extensible by apps and orgs?
- How is class *attached* to a datum and preserved across copy, transform,
  serialize, and IPC so it cannot be laundered away?
- What is a "purpose," who defines the vocabulary, and how is a holder prevented
  from using data acquired for one purpose to serve another?
- How does purpose flow through *derived* data — if app state is computed from
  location, does the result carry location's constraints?

## Omega Leverage

- **Data classes** map onto **domains** over `data` (`Datum::Health`,
  `Datum::Contact`) — named, provable predicates rather than runtime tags.
- **Purpose-scoped access** is a **typed capability** carrying both the class and
  the purpose; the authority-flow report then shows not just *what* data flows but
  *for what*.
- **`wire data`** preserves class/purpose annotations across boundaries.
- This is the chapter with the clearest **Omega gap**: capabilities are values +
  domains today, but *purpose-tagged authority* — a purpose that rides along a
  capability and constrains its use and propagation — is something Omega likely
  needs to grow for Cathedral to enforce it rather than merely record it.

## Open Questions

- Can purpose be *enforced* by the type system, or only *audited* after the fact?
  The privacy value differs enormously between the two.
- How is class assigned to data that arrives from outside (network, import, legacy
  box) with no inherent classification?
- Who adjudicates when an org's data-use policy conflicts with a user's privacy
  preference over the same datum ([[31_multi_user_and_org_control]])?

## Related
- [[03_capability_model]] — purpose-scoped reads are capabilities.
- [[06_security_policy_and_sandboxing]] — dangerous combinations range over classes.
- [[18_filesystem_as_database]] — where classified data actually lives.
- [[34_audit_compliance_provenance]] — purpose makes "why was this read?" answerable.
