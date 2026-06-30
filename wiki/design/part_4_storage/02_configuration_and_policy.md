# Chapter 02: Configuration & Policy

> Configuration is typed, owned, capability-scoped data across distinct homes — not one global mutable namespace — and policy is an orthogonal ceiling layered over it.

## The Legacy Model

Configuration on a mainstream OS is a junk drawer: `/etc` text files in a dozen ad-hoc syntaxes, the Windows registry, plist blobs, dotfiles, environment variables, command-line flags, and per-app databases nobody can enumerate. Nothing is typed, so an invalid value is discovered at runtime — if you are lucky, with a log line; if not, with a silent default. There is no schema, no validated migration when a key's meaning changes, no diff, no rollback, no audit of who changed what, and no coherent story for layering user vs. organization vs. device vs. application defaults. Secrets get pasted into the same files as ordinary settings. Answering "what is the effective value of X here, and why?" means reading source and guessing precedence.

Two mechanisms concentrate the lunacy. **Environment variables** are an ambient, inherited, untyped, globally-readable string namespace — a secrets-exfiltration vector (`/proc`, crash dumps, child inheritance) where you "just update a string and try not to fuck yourself." **The registry** is the same disease made persistent: a single mutable tree conflating kernel config, app config, security policy, and file associations, any entry of which can brick the machine if fat-fingered.

## The Cathedral Model

Configuration is not one store. It decomposes by **who owns the setting and who consumes it**, and "policy layering" is the *orthogonal* application of the same ceiling-intersection that governs all authority. What is uniform across every setting is that it is **typed** (domains), **versioned** (migration), **capability-scoped** (read and write are distinct authorities), and **auditable** (changes are transactional graph edges). What *differs* is where the value lives and how it is read. There is no global config namespace, no `/etc`, no registry — and environment variables and the registry simply do not exist natively.

## The decided mechanism

### Environment variables and the registry die

They are the same anti-pattern — a global, ambient, untyped, mutable string namespace — and Cathedral negates each property by construction: **ambient** → nothing is inherited without being handed it; **untyped** → typed `data` with domains; **global flat namespace** → every setting has an owner; **mutable by anyone** → capability-scoped writes; **silent failure** → a commit-time domain check; **a secrets vector** → secrets route out to operation-capabilities. Every legitimate job they did has a real home, so nothing is lost:

| Legacy use | Cathedral home |
|---|---|
| `HOME` — where's my data | the realm-root capability handed at session mint |
| `PATH` — find an executable | capability-mediated launch (resolve by identity/manifest) |
| `LD_LIBRARY_PATH` — find libraries | the content-addressed closure (resolve by hash) — the hijack vector dies with it |
| `LANG` / theme / accessibility | the shared preferences service |
| `AWS_SECRET_KEY`, `DATABASE_URL` | operation-capabilities ([[secrets_and_keys]]) — never bytes in a string |
| `DEBUG=1`, `RUST_LOG=trace` — flags | typed launch parameters / a scoped override lease |
| `PORT`, `REDIS_URL` — discovery | service capabilities (hold the flow, not a string address) |

The one legitimate need that survives — a parent handing a child startup parameters — becomes a **typed, explicit, non-inherited spawn argument** ([[component_model]]). Legacy code that calls `getenv()` or reads a registry key gets a **synthesized projection** over the real typed settings (the recursive-provider trick — the same way a legacy box gets a fake `/etc`), so killing them costs no compatibility. And you cannot brick the machine by editing a key: there is no single mutable tree, and boot-critical settings are not editable data at all — they are the measured TCB ([[boot_and_trust_chain]]).

### Config has five homes, and policy is an orthogonal sixth

A setting lives in exactly one place determined by its owner:

| Setting kind (example) | Owner | Mechanism | How a consumer reads it |
|---|---|---|---|
| **Resource tunable** (scheduler weight, energy/egress budget) | a trusted-core resource owner | the **value is a capability parameter** — it rides inside the cap you hold, not a store | you *hold* it; the resource owner enforces it |
| **Device/hardware setting** (brightness, resolution) | the device driver/service | **service-owned state, set via an operation-capability over IPC**, persisted in the service's realm | invoke a read/subscribe operation on the service |
| **Per-app preference** (theme, keybinds) | the app, in its own realm | **private realm data** — no IPC, no sharing | the app reads its own realm directly |
| **Cross-cutting user preference** (locale, light/dark, accessibility) | the user-Matrix | a **shared preferences service** — a scoped read+subscribe capability over a user-pref realm | read/subscribe through the service (+ change notifications) |
| **Secret-valued setting** (API key, token) | the secrets subsystem | **routes out to [[secrets_and_keys]]** as an operation-capability — never inlined | hold the operation, never the bytes |

**Organization/device policy is the orthogonal sixth thing — a ceiling, not a store.** A governing authority holds a **scoped ceiling capability** over a key, cap, or service, and it *clamps* any of the five homes at set-time. It is the ceiling-intersection from [[security_policy_and_sandboxing]] and [[multi_user_and_org_control]], applied per-key, never a fifth namespace. (Two things that look like config but are not app-config: **boot/firmware settings** are the static measured TCB, and `Config::Valid`-style domains enforce type-validity *within* each home.)

### Precedence is two operators, total and explainable

Layered resolution is not "highest layer wins." It is two distinct relationships:

- **inherit** (fill-absent): an unset key falls through to the next layer, terminating at a built-in default — well-ordered, so resolution is **total by construction** (the zero value is the identity element of precedence);
- **constrain** (cap/pin): a governing authority imposes a *ceiling* — a clamp, not a default.

The effective value is: resolve *up* the inheritance chain, then *clamp* to any governing ceiling. It is **explainable for free** because each layer's contribution is an authorized write recorded in the authority graph, so "X is 5 because device policy pins it" is a query, not a guess.

**A cap is a typed domain, and a violating override is rejected at commit.** A cap is a typed predicate over the key's domain (`Config::OrgAllowed` — a range, an enum subset, a pin). A user write outside it **fails at commit** with an explainable error ("rejected: org caps this at ≤ 8"), via the transaction's domain gate — never silently ignored or silently clamped. A pin shadows the user layer (not writable); a range cap rejects by default (clamp is an opt-in cap mode).

**Config versus app-state is no bright line — it is a role.** Both are typed versioned realm data, stored and migrated identically. A key is "config" when it plays the role of being *layered and legible*, not because it lives in a separate store; the same key can be both (an app default an org can pin).

**Multi-authority is the ceiling-intersection, and irreconcilable conflict is surfaced, not invented.** Usually two authorities (employer and school) govern *separate Matrices*, so they never touch the same key (structural isolation). Where both govern one shared key, ceilings **intersect — most-restrictive wins** (employer ≤ 80 ∧ school ≤ 60 → ≤ 60), total for compatible caps. Two irreconcilable *pins* (employer pins A, school pins B) give an empty intersection, and the OS **surfaces the conflict — it does not fabricate a winner.** That honesty is the answer; there is no arbitration algebra.

**The effective value is a derived, cached-with-invalidation view; the layers are the source of truth.** Semantically recompute-on-read; practically cache the materialized value and invalidate when a contributing layer commits. The cache is never authoritative (re-derivable from the layers and the resolution function), so audit and rollback operate on the **layers**. The resolution *function* is static and total; its *inputs* are live — which is why precedence is a runtime computation over live org/user state with a static, explainable algorithm.

## Concerns & Design Space

- **Declarative typed config.** Settings as typed `data`; validity as domains, so illegal configurations are unrepresentable rather than merely discouraged.
- **Schema validation & migration.** When a key's shape or meaning changes, a versioned migration carries existing config forward ([[versioned_state_and_migration]]), rather than silently reinterpreting old values.
- **Temporary overrides.** Scoped, expiring overrides (a debug flag for one session) are leases ([[capability_lifecycle]]), not permanent edits.
- **Auditable changes, diff, rollback.** Every change records who/what/when and is reversible — the config store is a small versioned object graph ([[filesystem_as_database]]).
- **Search.** Effective-value and provenance queries across the config graph are a query over named, authorized layer-contributions.
- **Zero value.** A zero configuration takes the inherited defaults: an unset key falls through to the next layer rather than erroring or hiding a built-in, so a zeroed layer is the identity element of precedence resolution ([[omega_substrate]]).

## Key Questions

- **Precedence algebra — resolved:** two operators (inherit + constrain) over the ceiling-intersection; total by construction (inheritance terminates at a default) and explainable (provenance per authorized contribution).
- **Org caps rejected at commit — resolved:** a cap is a typed domain; a violating write fails the commit-time domain check with an explainable error, never a silent clamp or ignore.
- **Config vs application state — resolved:** no bright line; "config" is the layered-and-legible *role* over uniform typed realm data, not a separate store.
- **Legacy text/registry config — resolved:** a synthesized projection over the typed store (the recursive-provider pattern), read-mostly with validated write-back.

## Omega Leverage

- **Typed `data` + versioned data** are a near-direct fit: config is long-lived typed state whose shape evolves, with migrations carrying old values forward — see Omega [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- **Domains** express validity classes and policy states (`Config::Valid`, `Config::OrgAllowed`) so layering, caps, and validation are proof facts — see Omega [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **Capabilities as values** scope read vs. write per key, so config access is an authority-flow fact, not an ambient file permission ([[capability_model]]).
- **`wire data`** gives a stable cross-version encoding for config that must be exported, synced, or read by external tooling — see Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).

## Open Questions

- The shared preferences service's exact interface (key set, change-notification shape, attenuation granularity for who may read which prefs).
- Whether the irreconcilable-pins conflict should ever be auto-resolved by an explicit declared arbitration order, or always kicked to a human.

## Related
- [[capability_model]] — resource tunables are capability parameters; read/write are distinct capabilities.
- [[security_policy_and_sandboxing]] — org/device policy as the ceiling-intersection.
- [[multi_user_and_org_control]] — the governing authorities and separate-Matrix isolation.
- [[secrets_and_keys]] — secret-valued settings routed out, never inlined.
- [[component_model]] — the typed, non-inherited launch context that replaces env vars.
- [[transactions_and_consistency]] — config changes as atomic, rollback-able transactions.
- [[versioned_state_and_migration]] — typed migration when config schema changes.
- [[audit_compliance_provenance]] — who changed which config, when, and why.
