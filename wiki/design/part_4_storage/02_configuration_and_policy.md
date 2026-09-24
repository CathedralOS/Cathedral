# Chapter 02: Configuration & Policy

> Configuration is typed, owned, capability-scoped data living in distinct homes, not one global mutable namespace. Policy is a ceiling layered over those homes, not another store.

## The Legacy Model

Configuration on a mainstream OS is a junk drawer: `/etc` text files in a dozen ad-hoc syntaxes, the Windows registry, plist blobs, dotfiles, environment variables, command-line flags, and per-app databases nobody can enumerate. Nothing is typed, so an invalid value is discovered at runtime. If you are lucky it is a log line; if not, a silent default. There is no schema, no validated migration when a key's meaning changes, no diff, no rollback, no audit of who changed what, and no coherent story for layering user, organization, device, and application defaults. Secrets get pasted into the same files as ordinary settings. Answering "what is the effective value of X here, and why?" means reading source and guessing precedence.

Two mechanisms concentrate the lunacy. Environment variables are an ambient, inherited, untyped, globally-readable string namespace. They are a secrets-exfiltration vector (`/proc`, crash dumps, child inheritance) where you "just update a string and try not to fuck yourself." The registry is the same disease made persistent: a single mutable tree conflating kernel config, app config, security policy, and file associations, any entry of which can brick the machine if fat-fingered.

## The Cathedral Model

Configuration is not one store. It decomposes by who owns the setting and who consumes it, and policy layering is the same ceiling-intersection that governs all authority, applied per key. Every setting is typed (domains), versioned (migration), capability-scoped (read and write are distinct authorities), and auditable (changes are transactional graph edges). What differs between settings is where the value lives and how it is read. There is no global config namespace, no `/etc`, and no registry. Environment variables and the registry do not exist natively.

## The decided mechanism

### Environment variables and the registry die

Environment variables and the registry are the same anti-pattern, a global, ambient, untyped, mutable string namespace, and Cathedral negates each property by construction:

- **Ambient.** Nothing is inherited without being handed it.
- **Untyped.** Settings are typed `data` with domains.
- **Global flat namespace.** Every setting has an owner.
- **Mutable by anyone.** Writes are capability-scoped.
- **Silent failure.** A commit-time domain check rejects an invalid value.
- **A secrets vector.** Secrets route out to operation-capabilities.

Every legitimate job they did has a real home, so nothing is lost:

| Legacy use | Cathedral home |
|---|---|
| `HOME`: where my data is | the realm-root capability handed at session mint |
| `PATH`: find an executable | capability-mediated launch (resolve by identity/manifest) |
| `LD_LIBRARY_PATH`: find libraries | the content-addressed closure (resolve by hash); the hijack vector dies with it |
| `LANG`, theme, accessibility | the shared preferences service |
| `AWS_SECRET_KEY`, `DATABASE_URL` | operation-capabilities ([[secrets_and_keys]]), never bytes in a string |
| `DEBUG=1`, `RUST_LOG=trace`: flags | typed launch parameters, or a scoped override lease |
| `PORT`, `REDIS_URL`: discovery | service capabilities (hold the flow, not a string address) |

The one legitimate need that survives is a parent handing a child startup parameters. That becomes a typed, non-inherited spawn argument ([[component_model]]). Legacy code that calls `getenv()` or reads a registry key gets a **synthesized projection** over the real typed settings, read-mostly with validated write-back. This is the recursive-provider pattern, the same way a legacy box gets a fake `/etc`, so killing them costs no compatibility. You also cannot brick the machine by editing a key. There is no single mutable tree, and boot-critical settings are not editable data at all; they are the measured TCB ([[boot_and_trust_chain]]).

### Config has five homes, and policy is an orthogonal sixth

A setting lives in exactly one place, determined by its owner:

| Setting kind (example) | Owner | Mechanism | How a consumer reads it |
|---|---|---|---|
| Resource tunable (scheduler weight, energy/egress budget) | a trusted-core resource owner | the value is a capability parameter; it rides inside the cap you hold, not in a store | you hold it; the resource owner enforces it |
| Device/hardware setting (brightness, resolution) | the device driver/service | service-owned state, set via an operation-capability over IPC, persisted in the service's realm | invoke a read/subscribe operation on the service |
| Per-app preference (theme, keybinds) | the app, in its own realm | private realm data; no IPC, no sharing | the app reads its own realm directly |
| Cross-cutting user preference (locale, light/dark, accessibility) | the user-Matrix | a shared preferences service: a scoped read+subscribe capability over a user-pref realm | read/subscribe through the service, with change notifications |
| Secret-valued setting (API key, token) | the secrets subsystem | routes out to [[secrets_and_keys]] as an operation-capability, never inlined | hold the operation, never the bytes |

Organization and device policy is the sixth thing, and it is a ceiling, not a store. A governing authority holds a **scoped ceiling capability** over a key, capability, or service, and it clamps any of the five homes at set-time. It is the ceiling-intersection from [[security_policy_and_sandboxing]] and [[multi_user_and_org_control]], applied per key, never a fifth namespace. Two things look like config but are not app-config. Boot and firmware settings are the static measured TCB. `Config::Valid`-style domains enforce type-validity within each home.

### Precedence is two operators, total and explainable

Layered resolution is not "highest layer wins." It is two distinct relationships:

- **inherit** (fill-absent): an unset key falls through to the next layer, terminating at a built-in default. The chain is well-ordered, so resolution is total by construction, and the zero value is the identity element of precedence.
- **constrain** (cap/pin): a governing authority imposes a ceiling. A ceiling is a clamp, not a default.

The effective value is resolved up the inheritance chain, then clamped to any governing ceiling. It is explainable for free, because each layer's contribution is an authorized write recorded in the authority graph, so "X is 5 because device policy pins it" is a query, not a guess.

A cap is a typed predicate over the key's domain (`Config::OrgAllowed`: a range, an enum subset, a pin), and a violating override is rejected at commit. A user write outside the cap fails at commit with an explainable error ("rejected: org caps this at ≤ 8") via the transaction's domain gate. It is never silently ignored and never silently clamped. A pin shadows the user layer, which becomes not writable. A range cap rejects by default; clamping is an opt-in cap mode.

Config versus app-state is a role, not a bright line. Both are typed versioned realm data, stored and migrated identically. A key is "config" when it plays the role of being layered and legible, not because it lives in a separate store. The same key can be both, such as an app default an org can pin.

Multi-authority resolution is the ceiling-intersection, and an irreconcilable conflict is surfaced rather than invented. Usually two authorities (employer and school) govern separate Matrices, so they never touch the same key; the isolation is structural. Where both govern one shared key, the ceilings intersect and the most restrictive wins (employer ≤ 80 ∧ school ≤ 60 → ≤ 60). This is total for compatible caps. Two irreconcilable pins (employer pins A, school pins B) give an empty intersection, and the OS surfaces the conflict. It does not fabricate a winner, and there is no arbitration algebra.

The effective value is a derived view, cached with invalidation, and the layers are the source of truth. Semantically it is recomputed on read; in practice the materialized value is cached and invalidated when a contributing layer commits. The cache is never authoritative, since it is re-derivable from the layers and the resolution function, so audit and rollback operate on the layers. The resolution function is static and total; its inputs are live. Precedence is therefore a runtime computation over live org and user state with a static, explainable algorithm.

### Config is authored by augmenting machines, not a file format

There are no config files: no TOML, YAML, INI, or dotfile grammar anywhere in the system. A settings object is a typed `data` value, populated by an **augmenting machine**, `machine configure(settings: &mut Settings)`, that sets the fields it cares about and leaves the rest at their ZII defaults. This is the same mechanism as the build manifest. Omega's `build.omg` is an augmenting machine over a `Build`, and config is that pattern generalized (see Omega's build and package model brief). It exists for the same reason: a bespoke config grammar is a secret export language, untyped, unvalidated, and a battle to comply with. A machine is ordinary typed Omega with real errors, no invented syntax, and full expressiveness, so a value can be computed from context rather than looked up in a string table.

Layering is augmenting machines called in order: org defaults, then user, then local, each mutating the same settings value. That is the `inherit` operator realized. Augmenting fills what is absent, so an untouched field falls through to the previous layer's write and ultimately to ZII. `constrain` is a ceiling machine applied after. The machines are effect-free, because mutating a passed-in local is not an effect, so the resolver runs them and reads the settings back. Config stays authored as code and consumed as data, and each machine is a named, authorized contribution in the authority graph, which is the provenance behind "explainable for free" above.

The settings type is owned by whoever the config belongs to. A service exposes its `Settings`, the platform exposes the ones in `contracts/`, and an app defines its own. A `configure` machine imports the type it augments from that owner. Which owner, and how the type evolves, is ordinary dependency detail that shifts as the system grows. The invariant is only that config is a typed value augmented by a machine, never a parsed format.

## Concerns & Design Space

- **Declarative typed config.** Settings are typed `data` and validity is a domain, so illegal configurations are unrepresentable rather than merely discouraged.
- **Schema validation & migration.** When a key's shape or meaning changes, a versioned migration carries existing config forward ([[versioned_state_and_migration]]) rather than silently reinterpreting old values.
- **Temporary overrides.** Scoped, expiring overrides (a debug flag for one session) are leases ([[capability_lifecycle]]), not permanent edits.
- **Auditable changes, diff, rollback.** Every change records who, what, and when, and is reversible. The config store is a small versioned object graph ([[filesystem_as_database]]).
- **Search.** Effective-value and provenance queries across the config graph are a query over named, authorized layer-contributions.
- **Zero value.** A zero configuration takes the inherited defaults. An unset key falls through to the next layer rather than erroring or hiding a built-in, so a zeroed layer is the identity element of precedence resolution ([[omega_substrate]]).

## Key Questions

- Can the system always answer "what is the effective value of X here, and why?" with a recorded provenance chain rather than a guess? This chapter is the one most accountable to that question.
- Which settings types does the platform own in `contracts/`, which belong to services and apps, and how does a settings type evolve without breaking the `configure` machines that import it?

## Omega Leverage

- **Typed `data` + declared historical shapes** are a near-direct fit: config is long-lived typed state whose published format evolves through immutable declarations and checked conversions. See Omega [Historical Data And Component Replacement](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#versioned-data).
- **Domains** express validity classes and policy states (`Config::Valid`, `Config::OrgAllowed`) so layering, caps, and validation are proof facts. See Omega [Domains](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_8_domains.md).
- **Capabilities as values** scope read versus write per key, so config access is an authority-flow fact, not an ambient file permission ([[capability_model]]).
- **Ordinary numbered data plus a selected codec** gives stable cross-version encoding for config that must be exported, synced, or read by external tooling. See Omega [Wire Protocols](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#wire-protocols).
- **Build-time evaluation + the complete normalized contract** make config-as-a-machine safe. The concrete `configure` invocation must fit the evaluator's floor: empty service reach, total, no unresolved failure/trap/abort route, and no blocking, suspension, or ambient authority. The reach row is necessary but not sufficient. The result is typed data rather than an opaque script. See Omega's build and package model brief and its build-time evaluation brief.

## Open Questions

- The shared preferences service's exact interface (key set, change-notification shape, attenuation granularity for who may read which prefs).
- Whether the irreconcilable-pins conflict should ever be auto-resolved by a declared arbitration order, or always kicked to a human.

## Related
- [[capability_model]] — resource tunables are capability parameters; read/write are distinct capabilities.
- [[security_policy_and_sandboxing]] — org/device policy as the ceiling-intersection.
- [[multi_user_and_org_control]] — the governing authorities and separate-Matrix isolation.
- [[secrets_and_keys]] — secret-valued settings routed out, never inlined.
- [[component_model]] — the typed, non-inherited launch context that replaces env vars.
- [[transactions_and_consistency]] — config changes as atomic, rollback-able transactions.
- [[versioned_state_and_migration]] — typed migration when config schema changes.
- [[audit_compliance_provenance]] — who changed which config, when, and why.
