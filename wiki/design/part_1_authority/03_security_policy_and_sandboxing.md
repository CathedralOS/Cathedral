# Chapter 03: Security Policy & Sandboxing

> Sandboxing is not a feature to add — it is what the capability model already produces. This chapter owns the policies that ceiling the authority graph.

## The Legacy Model

On legacy systems, isolation is a stack of afterthoughts bolted onto ambient uid power. A process starts able to do almost anything its uid can do, and then we *claw authority back*: seccomp filters, AppArmor and SELinux profiles, sandbox profiles, namespaces, cgroups, entitlement lists. Each mechanism speaks a different language, none of them sees the others, and all of them fight the default — which is that the process already had the power. The system cannot answer the questions that matter per permission: *why* does this app have camera access, *who* granted it, can it *store* it, can it *pass it onward*, can it use it *in the background*, and — the dangerous one — can it *combine* it with the network. Photo-read plus network is a different animal than either alone, and no legacy policy engine reasons about the conjunction.

## The Cathedral Model

Sandboxing falls out of [[capability_model]] for free: a component that was never handed a capability cannot use that authority, so the *default* is the sandbox. Policy is then not a wall built around a powerful process; it is a set of **ceilings** over an already-minimal, already-visible authority flow. A policy says, at most: which capabilities a principal may *hold*, which it may *store* or *delegate*, which **effects** it may reach, and which **boundary providers** may satisfy those effects.

The headline requirement is handling **dangerous combinations** — `Read<Photo>` ∧ `network_io` is a different animal than either alone. This was first imagined as a formal *policy language* that evaluates conjunctions of authority (the example below). **That formal engine is demoted** — see "The decided mechanism": deciding *danger* formulaically is intractable (intent is not in the capability set), so the conjunction ceiling is enforced by containment + judged real-grants + LLM-advised legibility, not a rule engine. The example records the shape that was rejected:

```omega
// A ceiling can forbid a conjunction even when each grant alone is allowed.
deny Principal::App when
    holds Capability<Read<Photo>> and
    reaches effect network_io;
```

Every sandboxed surface — app, service, driver, secret access, clipboard, screenshot, audio, camera, filesystem, hardware, inter-app communication — is the same kind of node under the same kind of ceiling. There is no separate "sandbox engine," and there is no ambient authority for policy to claw back.

### The decided mechanism

**Formulaically deciding "dangerous combination" is a fake problem — drop the policy engine.** "Dangerous" is *intent*, and intent is not in the capability set: `Read<Photo>` ∧ `network_io` is a photo-backup app *or* spyware, and no rule distinguishes them because the difference is the human's *purpose*, which the graph does not contain (the same limit as "proof cannot prove non-malicious"). A formal dangerous-combinations rule language can only over-block (kill legitimate apps) or under-block (miss novel combinations). So the conjunction ceiling is **not** a danger-deciding language. It is enforced three ways, none formulaic:

**1. Containment defuses the combination by default — no judgment needed.** Do not reason about whether a combination is dangerous; *neuter it by containment*. An app is granted what it asks, but its rights resolve **inside a Matrix to synthetic/contained resources by default** (virtual realms, [[filesystem_as_database]]; the Blank/Curated fidelities of [[human_permission_ux]]): "Photos" is a blank or curated realm, "the network" is contained. So **"granted the general right" ≠ "given the real resource."** The app boots happy and cannot tell it is contained (Matrix indistinguishability), and `Read<Photo>` ∧ `network_io` is *harmless* because the photos are blank and the egress reaches nothing real. The conjunction is defused by **what the Matrix actually exposes upward — which defaults to nothing — not by a policy rule.**

**2. The *real* hole-punch is separate, per-resource, gestural, and judged.** Two tiers: granting the capability **type** (the app holds `Read<Photo>`, contained) is distinct from granting the **real resource** (the hole-punch up to your actual photo library). Real access is minted one resource at a time by a user gesture (the picker mints real read of *the file you picked*; a network hole-punch reaches *the peer you allowed*), so the dangerous combination only *exists* if **both real hole-punches are deliberately granted** — the rare, scrutinized case, never the default.

**3. The judgment "is this real combination sketchy?" is LLM-advised + human, not a formula.** At the one moment it matters — granting real access that completes a worrying combination — the legibility/scanner agent ([[human_permission_ux]]) flags it in context ("a calculator is asking for real photos *and* real outbound network — that is suspicious"), advisory, and the human decides. Judgment, by a person with an LLM's help, does the deciding a rule cannot.

So what survives is not a policy language but a **discipline**: contain by default, make each real hole-punch a deliberate per-resource gesture, and advise the judgment with the legibility agent. At most a **tiny hard-block backstop** exists for a handful of genuinely-never-legitimate combinations (e.g. global keystroke capture ∧ unconstrained network, no gesture override) — explicitly a floor, *not* the mechanism. And because containment is the default, **getting a judgment wrong is survivable**: the worst case of a mistaken real-grant is bounded by everything *else* still being contained.

**The principle still holds, restated honestly:** "do not co-locate real-sensitive-read ∧ real-unconstrained-egress in one principal" is real — it is the exfil ceiling, and it is *why the browser is a Matrix with network ∧ nothing-real* ([[networking]]). But it is enforced by **containment + judged real-grants**, never by a formula that decides danger. Enforcement is therefore at **hand-off / containment**, not at *use* (deny-at-use based on data provenance is information-flow control — the deferred fine-grained residual, [[data_model_and_privacy]]). The genuinely hard part is the **legibility UX** of the judged real-grant ([[human_permission_ux]]), not a missing policy engine.

**Verified exception: a proof can *lift* a conjunction ceiling for a cooperative app.** Deciding *danger* is intractable, but a *specific dangerous flow* — "does any photo-derived value reach a network sink?" — is a **concrete, formalizable data-flow fact** (unlike intent). So a cooperative Omega-native app can carry a **proof** that the flow is absent, and a checked proof **lifts that specific ceiling**: the app holds `Read<Photo>` ∧ `network` *for real*, the danger proven gone rather than contained-away. This is the **verified/static form of information-flow control** — stronger than runtime `Secret<T>` taint-tracking ([[data_model_and_privacy]]) because the disjointness is proven once at zero runtime cost. It is **opt-in**: prove it → hold both for real; don't → fall back to containment. It does *not* prove the app benign (intent, unprovable) — it proves *one channel closed*, exactly the formalizable part.

**Proof composes across IPC between proven components — it breaks only at an unproven node.** The dividing line is not intra-app vs cross-app; it is **proven-connected-graph vs unproven boundary.** The mechanism is **effect-ceiling composition**: an effect ceiling is already a per-component proof of reachable behavior-classes, and ceilings compose *downward* (a component delegates only effects it holds), so a spawned subtree is bounded by its root's ceiling. Source-side the app proves "photo-data exits only via channel C"; recipient-side the subtree behind C carries a ceiling that **excludes `network_io`** — and the two compose into an end-to-end proof that photos cannot reach the network across *any number of IPC hops*, because nothing in that subtree can touch the network regardless of the bytes it is handed. So "share photos over IPC to a component that writes them to disk" is **provably safe** when that subtree excludes network. The proof **stops at the first unproven/foreign node** (a legacy component, a separate adversarial app), and at a *capability* handoff rather than a *data* handoff (delegating `Read<Photo>` is an authority-graph question bounded by "may-pass-onward", not a flow proof).

**Cross-boundary chains proof cannot reach → surface + LLM-scan + containment.** Where the flow crosses into an unproven component (app A dumps your files to location X; app B reads X and holds egress), no single proof sees the composition. But the OS holds the **capability graph**, so it knows the *structure* for free (A writes X, B reads X + has network) even though it cannot prove the *flow*. Such chains are **surfaced** ("A puts your files where B, who has the network, can read them") and an **LLM security scan** (the legibility agent) deduces whether the chain is a likely attack — advisory, human-judged. Containment at the boundary still bounds what actually leaves. So: **proof for the formalizable intra-graph flow; judgment (LLM + human) for the unformalizable cross-boundary intent; containment under both.**

### The mediating Matrix: generic by default, dangerous in proportion to what it forwards

A Matrix is a host that mediates its children's capability requests (forward / synthesize / deny), and two facts settle how to build one safely.

First, **a generic data-driven mediator is not over-privileged**, even though its code could forward any resource class: an *instance* holds only what its parent **delegated to it at spawn**, so its blast radius is the delegated subset, not the code's generality. The "a configurable default Matrix would need every capability" worry conflates *what the code can forward* with *what the instance holds*. So the common case is one **generic, audited mediator** parameterized by config, with **attenuated delegation** giving each instance least authority — no bespoke Matrix compiled per sandbox. Runtime Matrix compilation is at most a hot-path **JIT specialization**, never a requirement; a *specialized* Matrix (a container shipped with a known app) instead gets a tight **static manifest** at *package* time, and a *bespoke* Matrix (a debugger, a custom wrapper) is just an app declaring its own — possibly broad, honestly — manifest.

Second, **a Matrix's danger scales with the *real* authority it forwards, not with being a Matrix.** One that **synthesizes** its children's resources (blank realms, a contained network) holds almost nothing real — compromising it yields fakes, a low-value target. One that **forwards** real capabilities (a thin pass-through hosting all your apps) has all that authority flowing through it — compromise is catastrophic, a high-value target. So the defenses: keep the mediator **small and audited**, **prefer synthesis over forwarding**, and **attenuate** what does forward. The scary case is not "a Matrix" but "a thin forwarding Matrix in the path of everything"; an individual app holding only its own grants is contained by construction.

**Grant fulfillment is the host's prerogative.** When a child requests a resource, *how* it is satisfied is the host's choice — auto-grant a held or contained resource, summon a picker, synthesize a fake (the Real / Curated / Blank fidelities of [[human_permission_ux]]), or **escalate to its own host**. The picker is one fulfillment strategy, not a privileged OS surface. A resource living *outside* the Matrix is satisfied by the **outer host minting a capability and delegating it down through the mediating Matrix** (which may attenuate or refuse) to the child — the same delegation chain, expressed at whatever granularity (a file, or a shared realm) each host holds authority over.

## Concerns & Design Space

- **Ambient-authority elimination.** The whole approach collapses if any surface (clock, network, root filesystem, device tree) remains ambient. The sandbox is only as good as the absence of ambient power.
- **The six per-permission questions.** For each held capability the OS must answer: why held, who granted, may it be stored, may it be passed onward, may it be used in the background, may it be combined with other authority.
- **Dangerous combinations.** Conjunction-aware policy is the core novelty: photo-read + network, microphone + background, location + storage. Ceilings must range over the *graph*, not over isolated grants.
- **Effect and boundary ceilings.** Beyond *which* objects, policy bounds *what kind* of behavior (`filesystem_io`, `network_io`, `device_io`) and *which provider* may implement it ([[driver_model]]).
- **Secrets, clipboard, capture, media devices.** Each is a capability surface, not a special case — clipboard read, screenshot, audio, and camera are grants with the same lifecycle ([[secrets_and_keys]], [[capability_lifecycle]]).
- **Inter-app communication.** IPC is capability transfer; the policy that bounds what an app may *do* must also bound what it may *hand to a peer* ([[ipc_and_service_invocation]]).
- **Containers are synthetic realms.** A container or sandbox is an app handed attenuated or fabricated realm roots ([[filesystem_as_database]]) plus a scoped capability set. That one mechanism replaces the Linux namespace, cgroup, and seccomp stack, and the sandbox is the default, because an app only ever has the roots and capabilities it was given.
- **Transparent sandboxing.** An app's entire reality is the capabilities it holds: every observation (files, devices, disk info, even time) routes through a host-owned provider, so a *sealed* sandbox is fabricable in software and undetectable to trusted-toolchain code. The only residue is raw CPU instructions a native (non-Omega) binary can execute directly, which need hardware trapping to virtualize ([[kernel_architecture]]); system-access confinement holds for any app in any language regardless.
- **Policy introspection & revocation UX.** A policy that cannot be read back, or a revocation a human cannot understand the consequences of, is not a policy ([[human_permission_ux]], [[observability_and_introspection]]).
- **Auditing.** Every grant, denial, and combination check is an event in the provenance record ([[audit_compliance_provenance]]).
- **Zero value.** A zeroed policy is the empty ceiling set over a principal that holds nothing (shape 1 in [[omega_substrate]]), which is exactly the deny-all default sandbox, so an uninitialized policy is the safest one rather than an open or ambient one.

## Key Questions

- For any app, can the OS produce the *why / who / stored? / passable? / background? / combinable?* answer for every permission it holds?
- ~~How are dangerous-combination rules expressed, and are they evaluated statically over authority flow, dynamically at grant time, or both?~~ **Resolved/demoted** (see "The decided mechanism"): there is no formal rule engine — deciding danger formulaically is intractable. The conjunction ceiling is enforced by containment-by-default + per-resource judged real-grants + LLM-advised legibility; judgment (human + LLM), not a formula, does the deciding.
- What is the relationship between a policy ceiling and the actual held authority — is the ceiling enforced at hand-off, at use, or at both? **At hand-off / containment** (use = data-flow = IFC = deferred).
- What does revocation *look like* to a human, and how is the consequence ("this will break X") computed before they confirm?

## Omega Leverage

- A policy is a set of **ceilings over inferred authority flow** — the accepts/uses/derives/stores report from [[capability_model]] is exactly the surface a ceiling bounds.
- **Effect ceilings** give the orthogonal axis: bound the behavior vocabulary a component may reach, independent of which objects it names.
- **Boundary-provider ceilings** bound *who* may implement an effect, so a policy can forbid an unapproved driver from satisfying `device_io`.
- Conjunction-aware denial (reasoning over *combinations* of held authority) is beyond today's per-effect/per-flow checks — Cathedral pushes a **policy language over the authority graph** onto the runtime as an extension.

## Open Questions

- Are combination rules global, per-publisher, per-org, or per-data-class ([[data_model_and_privacy]])? Who is authoritative when they conflict?
- Can "no dangerous combination is reachable" ever be proven statically, or is it inherently a runtime graph query?
- How are policy ceilings versioned and migrated when an app updates and its authority footprint changes ([[updates_and_hot_swap]])?
- Capability confinement of system access holds for any app in any language, but a native app reads raw CPU state (`rdtsc`, `cpuid`) directly. So can a sandbox be made *undetectable* to native code at all without hardware trapping/virtualization, or is the host-floor for transparency a hardware feature ([[kernel_architecture]])?

## Related
- [[capability_model]] — policy is ceilings over this graph.
- [[data_model_and_privacy]] — data classes the combination rules range over.
- [[human_permission_ux]] — how ceilings and revocation are shown to humans.
- [[observability_and_introspection]] — reading policy and authority back.
