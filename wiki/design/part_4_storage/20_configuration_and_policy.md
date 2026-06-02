# Chapter 20: Configuration & Policy

> Configuration is typed, versioned, capability-scoped data — not text-file
> archaeology — and policy is the layered resolution of that data across actors.

## The Legacy Contract

Configuration on a mainstream OS is a junk drawer: `/etc` text files in a dozen
ad-hoc syntaxes, the Windows registry, plist blobs, dotfiles, environment
variables, command-line flags, and per-app databases nobody can enumerate.
Nothing is typed, so an invalid value is discovered at runtime — if you are lucky,
with a log line; if not, with a silent default. There is no schema, no validated
migration when a key's meaning changes, no diff, no rollback, no audit of who
changed what, and no coherent story for layering user vs. organization vs. device
vs. application defaults. Secrets get pasted into the same files as ordinary
settings. Answering "what is the effective value of X here, and why?" means
reading source and guessing precedence.

## What Cathedral Wants

A single typed configuration substrate. Every setting is a field in a typed,
versioned `data` shape with declared validity; writing an invalid value is a
compile- or commit-time error, not a runtime surprise. Configuration *layers* —
user, organization, device, application defaults — resolve through an explicit,
inspectable policy precedence, so the effective value and the layer it came from
are always a query. Changes are transactional ([[19_transactions_and_consistency]]),
diffable, rollback-able, and audited ([[34_audit_compliance_provenance]]).
Secrets are a separate, capability-gated namespace ([[07_secrets_and_keys]]), never
inlined. This is the enterprise wedge: configuration as versioned, typed,
capability-scoped data with auditable, layered policy.

## Concerns & Design Space

- **Declarative typed config.** Settings as typed `data`; validity as domains, so
  illegal configurations are unrepresentable rather than merely discouraged.
- **Schema validation & migration.** When a key's shape or meaning changes, a
  versioned migration carries existing config forward ([[21_versioned_state_and_migration]]),
  rather than silently reinterpreting old values.
- **Policy layering.** Precedence across user / organization / device / app
  layers, with a defined, inspectable resolution rule ([[31_multi_user_and_org_control]]).
  Organization policy can *pin* or *cap* a value the user may otherwise set.
- **Temporary overrides.** Scoped, expiring overrides (a debug flag for one
  session) modeled as leases ([[04_capability_lifecycle]]), not permanent edits.
- **Auditable changes, diff, rollback.** Every change records who/what/when and
  is reversible — the config store is itself a small versioned object graph
  ([[18_filesystem_as_database]]).
- **Capability-scoped read/write.** Who may *read* a config key and who may *write*
  it are distinct capabilities ([[03_capability_model]]); a component sees only the
  config it holds authority over.
- **Secrets separation.** Secret-valued settings resolve through the key
  subsystem and never materialize as plaintext in the config store
  ([[07_secrets_and_keys]]).
- **Search.** Effective-value and provenance queries across the whole config graph.

## Key Questions

- What is the precedence algebra for layered policy, and is it total (always a
  defined winner) and explainable ("X is 5 because device policy pins it")?
- How are organization-imposed caps expressed so a user override is rejected at
  commit rather than silently ignored?
- Where does config end and application state begin — is there a bright line, or
  a spectrum the substrate must handle uniformly?
- How do unmanaged legacy apps ([[18_filesystem_as_database]] compatibility) read
  config they expect as text files, projected from the typed store?

## Omega Leverage

- **Typed `data` + versioned data** are a near-direct fit: config is exactly
  long-lived typed state whose shape evolves, with migrations carrying old values
  forward — see Omega [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- **Domains** express validity classes and policy states (`Config::Valid`,
  `Config::OrgPinned`) so layering and validation are proof facts — see Omega
  [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **Capabilities as values** scope read vs. write per key, so config access is an
  authority-flow fact, not an ambient file permission ([[03_capability_model]]).
- **`wire data`** gives a stable cross-version encoding for config that must be
  exported, synced, or read by external tooling — see Omega
  [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).

## Open Questions

- Can the full layered precedence be resolved statically for a known device, or
  is it inherently a runtime computation over live org/user state?
- How are conflicting org policies from multiple authorities (employer + school)
  composed — first-writer, most-restrictive, or explicit arbitration?
- Should the effective configuration be materialized and cached, or always
  recomputed from layers on read, given audit and rollback requirements?

## Related
- [[19_transactions_and_consistency]] — config changes as atomic, rollback-able transactions.
- [[21_versioned_state_and_migration]] — typed migration when config schema changes.
- [[31_multi_user_and_org_control]] — the policy layers and who owns each.
- [[07_secrets_and_keys]] — secret-valued settings kept out of the config store.
- [[34_audit_compliance_provenance]] — who changed which config, when, and why.
