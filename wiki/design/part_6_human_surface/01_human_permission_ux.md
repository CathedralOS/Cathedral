# Chapter 01: Human Permission UX

> The hardest part of a capability OS is not the type system — it is the human gesture. This chapter owns how people *mint, delegate, and revoke* authority through interface actions they already understand.

## The Legacy Model

Mainstream permission UX is a tax bolted onto an ambient-authority OS. Either the app already has broad access (desktop: read any file the user can), or the user is interrogated by a stream of modal prompts ("App X wants to access your Photos") that they click through blind. Prompts are vague, un-attributed, easy to spoof, and grant *categories* ("all photos") rather than *objects* ("this photo"). Revocation is buried in a settings panel nobody visits, and "what does this app currently have?" has no honest answer. The deepest failure: the file picker — the single most important permission UI ever built — is treated as a mere file-chooser dialog, its authority-granting nature completely unmodeled.

## The Cathedral Model

Make the human gesture the *primary* way authority is minted and moved. The **key insight**: in a capability OS the file picker is not a dialog — it is an **authority mint**. When the user picks `report.pdf`, the OS mints a narrow `Capability<File::Read("report.pdf")>` for exactly that object and delegates it to the requesting app. Nothing broader is created. The same principle generalizes:

- **Drag-and-drop** — dragging an object delegates a capability over *that object* to the drop target.
- **Share sheet** — selecting a recipient delegates a one-shot capability over the shared payload.
- **"Open with"** — choosing a handler mints a capability for the chosen file and delegates it to the chosen app.

Each is a *principled authority-transfer mechanism*: the user's intent, expressed as a gesture, mints or attenuates a narrow capability for exactly the chosen object (see [[capability_lifecycle]] for minting/delegation/attenuation). No ambient grant, no category-wide blanket. Prompts become rare and meaningful; most authority flows through gestures the user already performs for other reasons.

These gestures must run on the compositor's **trusted path** ([[windowing_and_compositor]]) so the picker, the share target, and the app identity shown cannot be spoofed.

### What the model actually fixes

Permission UX gets judged on three separate axes, and conflating them is why the comparison feels muddy:

| Axis | Windows (UAC) | macOS (TCC) | Cathedral |
|---|---|---|---|
| Un-spoofable surface | Strong (secure desktop) | Weak (system-drawn but occludable) | Strong, without a full takeover |
| Who initiates | The app | The app | The user's gesture; a prompt only when no gesture fits |
| Scope of the grant | Blanket administrator | A category ("all photos") | The one object named |
| Tied to user intent | No | No | Yes, the action is the grant |
| CLI inheritance | Elevate once, children inherit everything | `sudo`: once, then inherited | No ambient authority to inherit |

Two corrections fall out of this. A trusted path does not require Windows's screen-dimming takeover; it requires an OS-drawn surface the compositor guarantees is focused and un-occludable, so the actual takeover is reserved for session-level events like login ([[sessions_and_login]]). And the app should not be the one initiating: the common case is the user's own gesture minting a scoped grant, with a prompt only where no natural gesture exists.

### Synthetic grants: defeating permission coercion

Apps weaponize permissions ("no full photo access, no launch"). The capability model removes the *leverage* rather than fighting the app, because the answer to a demand need not be a real grant or a hard denial. A request resolves to one of three fidelities, and the everyday "deny" is the third:

- **Real** — the actual objects.
- **Only what I pick** — the curated subset, via the picker.
- **Blank** — a real-looking but empty realm ([[filesystem_as_database]]); the app's "do I have photos?" returns yes, it boots, and it sees nothing.

Because **Blank** is the default deny, an app can never distinguish "the user said no" from "the user has nothing," so gating on the grant extracts nothing and stops paying. A true hard-deny (the app is *told* it has no access) survives only as an advanced toggle for *cooperative* apps that should show a correct "grant access to enable X" prompt, not as a defense. One caveat: a blank *writable* slot sends the app's saves into its own overlay, so the UI must surface where those files went rather than letting them vanish.

### The world chooser

Sandboxing is the default ([[security_policy_and_sandboxing]]), so the UX is not a switch but a choice of the app's **world**: a few human slots, each at a fidelity. Right-click an app and "Run sandboxed…" offers Files / System / Network / Devices / Identity, each Real / Curated / Blank / None, with presets (Default, Locked down, Throwaway, Trusted). A live per-app panel shows what it holds and is using *right now*, with one-click revoke and the ability to flip a fidelity on the running app, because the runtime re-points the binding underneath it ([[filesystem_as_database]] resolution environment). Throwaway (discard on exit), clone, and reset come free, because a world is just capabilities plus roots plus an overlay — structurally, an object owning its world-realm, created by instantiating a template closure, browsable in the file browser, deleted by refcount-GC ([[security_policy_and_sandboxing]]). All of it is progressive disclosure: the everyday user only meets the picker and the prompt; the chooser is for when someone wants it, and it runs on the trusted path so an app cannot fake being sandboxed.

### Granting at the command line

A shell is the same minting model in text. It runs as the user's session principal ([[sessions_and_login]]), so it wields the user's own authority rather than a sandboxed app's, and typing a path is the gesture: when the user runs `cat ./report.pdf`, the shell resolves the path, mints `Capability<File::Read("report.pdf")>` for that one object, and passes it to `cat`. The arguments are the grants. `rm a.txt b.txt` hands `rm` a delete capability for each named file and nothing else.

A spawned command gets exactly three things: capabilities for the paths it was named on the line, the current directory as a scoped standing delegation (so it can work in the folder without naming every file), and whatever its own manifest declares and policy permits, such as network for a downloader. Nothing ambient beyond the cwd. "It needs everything" resolves to "the user can name anything they already hold," which is their own realm, scoped by what they actually type; naming `/` hands over the realm root the user already has, not the system realm or another user's.

This closes the `sudo`/UAC inheritance hole. The shell holds broad authority because it is the user, but it never passes that wholesale: each command gets only the slice its invocation named, so `rm a.txt` cannot also read the photos. Reaching the system realm is a per-operation trusted-path mint for the specific object, not a blanket elevated shell that every child inherits.

### Gesture mechanics: the picker's reach, drag, and the clipboard

**Even *asking* is gated, which makes minimum-authority the default.** The picker is itself reached through a capability: an app with no picker capability cannot invoke the picker *at all*, and a *narrower* one ("pick an image") **parameterizes** the picker so only conforming objects are selectable. On accept, the OS makes the grant entry and returns a handle for later redemption — picker-as-mint, mechanically a [[capability_lifecycle]] delegation. The consequence is the incentive: because each pick mints a *fresh per-use* grant, an app that uses the picker holds **zero standing file authority**, so "least authority" is the default path, not a virtue an app must choose. Over-asking only appears as a request for *standing/ambient* access — exactly where the scarier prompt and the visible manifest (a dev's "why does this library want the network?") apply the pressure. The residual hard problem is therefore not incentive but **legibility**: presenting held authority comprehensibly enough that scrutiny is informed rather than reflexive. (The picker can also honor a no-capture property — hidden from screen capture by default, with an in-dialog toggle for someone recording a tutorial.) Revoking a picked grant bumps its generation, so the next redemption fails as `CapabilityRevoked`, surfaced to the app as an ordinary IO error — like the file ceasing to exist.

**A drag re-mints a grant the source already holds.** Mouse-down on an object mints a **drag-op into the dragging principal's arena** — that arena slot *is* the staging, so a drag needs no separate buffer and a source that dies mid-drag simply cancels it. It carries **whatever grant the source derives and the target's manifest accepts**: `Copy` by default (the recipient gets its *own* cell on the same content-addressed body — an independent file, original untouched), `Read` (shared live access), `Move` (copy + the source relinquishes its cell), or `Write` (drag a file into an editor). It is **program-minted** — a file-holding app (the desktop, a file explorer: *ordinary replaceable apps with no privilege*) derives the narrow grant by attenuation (`Folder::Read` → `File::Copy` for the one object) and can never grant more than it holds. The OS/compositor is **courier and policy** only: it renders the drag-image and routes the drop; the authority is the source's derived grant, delivered into the target's arena on drop.

**Copy-vs-move is now explicit intent, not a topology guess.** The legacy "same-drive = move, cross-drive = copy" heuristic existed only because move was a cheap pointer-flip and copy an expensive byte-duplication. Content-addressing makes copy **always O(1)** (a new cell on a shared body), so there is no cost asymmetry to infer from — the choice collapses to "does the source keep its cell?", an explicit modifier or drop-menu. (Cross-*device* byte movement is the storage layer's relocation job, orthogonal to the gesture.)

**Across a Matrix boundary, the drag is re-minted by the owner.** A drag carries two things — the **resource handle** (the grant) and a **visual** (the drag-image). At a boundary the inner compositor **clamps the cursor at its edge**: from inside, the drag merely "hit the screen edge and is still held," and the inner Matrix *cannot perceive an outside* (confinement holds — it cannot even probe the boundary). The **owner/virtualizer** — the sole party that can mint across its own boundary, so no app can forge a cross-Matrix drag — sees the edge-hit-with-active-drag, checks its **per-direction policy** (`may_drag_in` and `may_drag_out` are independent, so one-way falls out naturally), and if permitted **mints a fresh drag in the adjacent frame carrying the same resource handle** while handing the visual to the adjacent compositor. So two things cross — authority (re-minted) and visual (cursor hand-off). Root closes its boundary (no drag user↔root); a user-owned sub-Matrix opens it however its owner chooses. Only **files / realm-object capabilities** cross a boundary — the only thing meaningful to hand a confined world; in-app dragging (reordering, a widget) never reaches the boundary and is the app's own concern.

**The clipboard is the OS-held *persistent* variant of a drag grant — a realm and a handle, never a RAM buffer.** Drag is source-held and transient; the clipboard outlives the source, so it lives where durable state lives. It is *filesystem-as-database again*: a realm the Matrix owner holds, containing **content-addressed copy grants**. Because the body is content-addressed it is copied in **O(1)** and, being immutable, yields a **stable snapshot for free** — paste survives the source editing its cell (new writes → a new body), dying, or a reboot, because the handle pins the old body by refcount. Propagation is the **same owner-gated nesting as drag**: a clipboard handle forwards up the Matrix tree only if each owner opts in, each level decides sibling fan-out, and root decides any cross-user visibility; the *handle* travels while the *content* is one body materialized at the root-most accepting backing (no per-level copies). Per-entry **retention** is the storage chapter's commit attribute: a copied password commits `wipe`/`transient` (crypto-erased, short-lived, never durably on disk), a normal copy `retain`. Format negotiation is the **target's** concern (it holds the object and converts), not a special clipboard format-list. A network clipboard is just the cross-*machine* capability case — possible via the serialized-capability path with all its partition caveats, but a fringe nicety, deferred.

### Action-confirm: minting authority for an irreversible act

The picker designates a passive *resource*. An irreversible *action* — pay, send, delete, sign, wipe — needs a different gesture, and payments are its hardest case, so they are the worked example here ([[secrets_and_keys]] owns the credential side).

**The gesture mints a one-shot, parameter-bound action capability.** The app holds *no* standing authority to act; it **requests** ("charge $48 to Acme"), the OS renders the exact action on the trusted path (one of the few places a full compositor takeover *is* justified, like login), and the user's confirm **mints `Charge(≤$48, payee=Acme, once)`**, redeemed once. Same picker-as-mint shape, for a verb instead of a noun.

**WYSIWYS is the keystone — the capability is minted from the *rendered* parameters, not the app's request.** What-you-see-is-what-you-sign: the surface's displayed amount and payee are exactly what the capability carries, so an app cannot show "$48" and redeem "$480". This is the hardware-wallet "verify on the device's own screen" principle; without it the confirm is theatre.

**Friction scales with stakes.** A $5 purchase is a tap; an irreversible high-value or trust-transitive act (a large wire, "wipe this device", granting *standing* authority) escalates to re-authentication, a delay, or an out-of-band confirmation — the staking tiers from the first-pin model ([network_trust_fabric](../speculation/network_trust_fabric.md)), applied to actions.

**For money, it is one signed chain — not a local grant plus a separate remote beg.** The links are continuous: the WYSIWYS confirm **unlocks the [[secrets_and_keys]] Warden's on-device key** → the Warden **signs a fresh cryptogram over `(amount, payee, nonce)`** → that signature *is* the remote authorization request → the issuer fraud-checks and, for a purchase, **authorizes-and-captures in one step** → the app sees "approved". The signature covering `(payee, nonce)` is what makes "$50 for X" un-replayable and un-rerouteable to "$50 for Y". A resend de-dupes on the **idempotency key**; a timeout is the **`Unknown`** outcome ([[error_model_and_recovery]]) → re-query status before retrying, never blind-retry.

**Auth → capture → settle is the output-commit pattern in a domain.** The remote **authorization** is a scoped, expiring, attenuable capability the *merchant* holds (one-shot for a purchase; standing-bounded with incremental draw for hotels/fuel/tips); **capture** is its attenuated, idempotent redeem (≤ the ceiling); **settlement** is the banks' asynchronous batched netting — invisible to the app, whose world ends at "approved". That is exactly [[transactions_and_consistency]]'s output-commit: commit the outbox on approval, treat settlement as the downstream irreversible effect.

**Agents get a ceiling'd lease — which is a virtual card.** An autonomous agent cannot tap-confirm each charge, so the human mints *once* a leased, ceiling'd action capability ("≤ $50 per charge, ≤ $200 this session, this merchant") — mechanically identical to a merchant-locked, amount-capped, revocable **virtual card** ([[agents_as_principals]]). The effect-ceiling *is* the spend limit.

**Reversibility is a service, not a primitive.** A one-shot push authorization is clean but *final*; the chargeback/dispute machinery card networks sell is a separate **adjudication layer**. If Cathedral's money system wants consumer protection, some component runs a dispute service over the ledger — it does not fall out of the capability primitive, and the design must not pretend it does.

## Concerns & Design Space

- **The picker as a mint.** A trusted OS-owned surface that returns a freshly minted, object-scoped capability — the app never sees the broader namespace.
- **Drag/drop, share, open-with** as the same authority-transfer primitive with different ergonomics; one underlying delegation model.
- **Revocation UI.** A first-class, discoverable surface answering "what does this app hold, from which gesture, and can I take it back?" — backed by the live authority graph ([[capability_model]]).
- **No blessed file manager.** The file browser is an *ordinary, replaceable app holding capabilities*, not an ambient "it is the user" carve-out. It holds a broad **enumerate** bundle (including `private` *iff* granted) to *show* you the tree, but **content reads are minted by your gesture** — double-click is a picker-style read-mint to the preview pane or the opening app — so even your own browser never *ambiently* reads your files ([[filesystem_as_database]]'s enumerate/read/write decomposition). Replace it, grant the replacement the same bundle, identical behavior; nothing is blessed.
- **Legibility is the hard residual.** A precise authority graph is not human-legible, and the failure mode of every permission system is reflexive approval. The levers: roll capability atoms up into human nouns (Photos, Documents, money); foreground *observed use* over the static grant; surface *anomalies*, not the steady state ("an app you haven't opened in months still holds your camera"); show revoke blast-radius; and trace each grant to the gesture that minted it. A **legibility agent** is the tooling: a read-only audit over the authority *graph* — never content, because **auditing authority ≠ exercising it** (it holds a read-only graph capability, not a decrypt-op) — run *locally*, *advisory not enforcing*, with an optional on-device LLM narrating "why is this risky?" at grant time. It is an ordinary replaceable app, not blessed, and a hostile app wanting its grant to read as benign is why anomaly-surfacing must resist gaming.
- **App identity & anti-spoofing.** Every grant prompt and picker authentically shows the requesting principal ([[windowing_and_compositor]], [[identity_and_principals]]); a hostile app cannot dress as a trusted one.
- **Background-task consent.** Authority for work the user is *not* watching (sync, location-in-background) should be a visible **lease**, renewed with consent, not a permanent grant.
- **Grants persist or lease by sensitivity.** Most everyday grants **persist** in the app's realm — a picked file or folder is a durable grant (you do not re-pick your home folder every launch), so the persisted grants *are* the cache. Sensitive device grants (camera, microphone) are **leases**, held while in use and re-confirmed on a timer or "while using." One-shot actions (a payment, a wipe) **never persist** — minted per-action via action-confirm. So re-asking is the sensitive/transient tail, not the default; the leasing model ([[capability_lifecycle]]) sets persistence per cap-type, defaulted by sensitivity.
- **The assistive tier.** Screen readers and switch input need the heaviest grant in the system: structure and input across everything, *including the grant prompts themselves*, plus named dispatch ([[windowing_and_compositor]]). Enrollment is its own deliberate, high-friction trusted-path flow — never a rote prompt — and the secure-entry carve-out (masked echo and focus position, never content) is part of the tier's definition, so accessibility survives in the one place coercion-resistant design would otherwise kill it.
- **Being watched is visible.** Every active observation grant — capture, structure, input, injection — carries an OS-chrome indicator no shell can hide; the indicator and the revocation surface are two views of the same live grants ([[windowing_and_compositor]]).
- **Purpose at the gesture.** A grant can carry a declared purpose ([[data_model_and_privacy]]) so later use is checkable against *why* it was given.
- **Least-surprise defaults.** When ambiguous, mint the narrowest plausible capability and let the user widen explicitly — never the reverse.
- **Synthetic grants vs. coercion.** "Blank" (a real-looking empty realm, [[filesystem_as_database]]) is the default deny, so an app cannot tell denial from emptiness and gating on the grant stops paying; hard-deny is an advanced toggle for cooperative apps only.
- **The world chooser.** Sandbox config presented as a few fidelity slots (Real / Curated / Blank / None) plus a live per-app revoke panel; throwaway, clone, and reset fall out of a world being capabilities-plus-roots ([[security_policy_and_sandboxing]]).
- **The command line is a textual mint.** The shell runs as the session principal and turns named paths into scoped capabilities passed to each command, so arguments are grants and there is no elevate-once-inherit-everything hole ([[sessions_and_login]]).
- **Trusted path without takeover.** Un-spoofability is an OS-drawn, un-occludable surface, not a screen-dimming event; full takeover is reserved for login and unlock ([[windowing_and_compositor]]).
- **Zero value.** The Blank grant is the ZII zero of a grant ([[omega_substrate]]): a zero grant is the capability over the canonical null realm, valid-empty rather than a hostile denial, so an app handed a zeroed grant boots and sees an empty world, which is why default-deny and zero-is-valid are the same value from two sides.

## Key Questions

- **Picker contract — decided** (see gesture mechanics): even *invoking* the picker is gated by a capability, and a narrow one parameterizes it (image-only); on accept the OS mints a fresh per-use object-scoped grant and returns a handle — the app never sees the surrounding namespace, and because each pick is per-use the app holds *zero standing* authority, which is what makes least-authority the default. Residual: **legibility** of held authority, not the contract.
- **Drag-and-drop across boundaries — decided** (see gesture mechanics): the **owner/virtualizer is the sole re-minter** at a Matrix boundary, so there is no spoofable intermediary — it clamps the cursor at the edge, checks a per-direction policy, and re-mints the same resource handle (plus the visual) into the adjacent frame; intra-Matrix, the drop mints the source-derived grant straight into the target's arena.
- What does the revocation surface show, and how does it map back onto delegation edges in the authority graph? *(Direction: the levers are noun-rollup, observed-use, anomaly-surfacing, revoke-blast-radius, and mint-provenance, with a local read-only **legibility agent** auditing the graph — see Concerns. The exact UI is deferred to implementation; the **pieces** — the enumerate/read/write decomposition, the graph, observed-use logs — are the design.)*
- How is background-task consent expressed as a lease the user can see and end?
- At the command line, how does the shell decide the cwd delegation and a command's declared needs without re-prompting per file, while still scoping each child to what its invocation named?

## Omega Leverage

- Picker, drop, and share results are **capabilities + domains** minted by a trusted broker and **delegated** to the app — exactly Omega's broker/acquire pattern ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Object-scoping is **attenuation** via a tighter [domain](../../../../Omega/wiki/language_guide/chapter_8_domains.md) (`Folder::Readable` → `File::Readable`).
- Background grants are **leases**, tied to trusted time and explicit renewal.
- The revocation surface reads the same authority-flow / graph data as [[capability_model]] — no separate bookkeeping.
- Omega does not define the *interaction* layer; the mint-on-gesture mapping is Cathedral's contribution, but it bottoms out in standard delegation.

## Open Questions

- Can the picker be made so reliable that category-wide grants ("all photos") are never needed — or are some workflows inherently broad?
- How much can we infer authority from gestures without a prompt before users feel the system is acting behind their back?
- Is "Blank" the right default for a pushy app, and how is a blank *writable* slot's saved data surfaced so the user is not confused about where it went?
- How does the live world panel present an app's fidelities and current usage without overwhelming a non-expert user?

## Related
- [[capability_model]] — the held authority these gestures mint and move.
- [[capability_lifecycle]] — minting, delegation, attenuation, revocation.
- [[security_policy_and_sandboxing]] — ceilings that bound what a gesture can grant.
- [[windowing_and_compositor]] — the trusted path the picker must run on.
- [[data_model_and_privacy]] — purpose carried alongside the granted capability.
