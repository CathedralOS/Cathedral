# Chapter 28: Human Permission UX

> The hardest part of a capability OS is not the type system — it is the human gesture. This chapter owns how people *mint, delegate, and revoke* authority through interface actions they already understand.

## The Legacy Contract

Mainstream permission UX is a tax bolted onto an ambient-authority OS. Either the app already has broad access (desktop: read any file the user can), or the user is interrogated by a stream of modal prompts ("App X wants to access your Photos") that they click through blind. Prompts are vague, un-attributed, easy to spoof, and grant *categories* ("all photos") rather than *objects* ("this photo"). Revocation is buried in a settings panel nobody visits, and "what does this app currently have?" has no honest answer. The deepest failure: the file picker — the single most important permission UI ever built — is treated as a mere file-chooser dialog, its authority-granting nature completely unmodeled.

## What Cathedral Wants

Make the human gesture the *primary* way authority is minted and moved. The **key insight**: in a capability OS the file picker is not a dialog — it is an **authority mint**. When the user picks `report.pdf`, the OS mints a narrow `Capability<File::Read("report.pdf")>` for exactly that object and delegates it to the requesting app. Nothing broader is created. The same principle generalizes:

- **Drag-and-drop** — dragging an object delegates a capability over *that object* to the drop target.
- **Share sheet** — selecting a recipient delegates a one-shot capability over the shared payload.
- **"Open with"** — choosing a handler mints a capability for the chosen file and delegates it to the chosen app.

Each is a *principled authority-transfer mechanism*: the user's intent, expressed as a gesture, mints or attenuates a narrow capability for exactly the chosen object (see [[04_capability_lifecycle]] for minting/delegation/attenuation). No ambient grant, no category-wide blanket. Prompts become rare and meaningful; most authority flows through gestures the user already performs for other reasons.

These gestures must run on the compositor's **trusted path** ([[27_windowing_and_compositor]]) so the picker, the share target, and the app identity shown cannot be spoofed.

## Concerns & Design Space

- **The picker as a mint.** A trusted OS-owned surface that returns a freshly minted, object-scoped capability — the app never sees the broader namespace.
- **Drag/drop, share, open-with** as the same authority-transfer primitive with different ergonomics; one underlying delegation model.
- **Revocation UI.** A first-class, discoverable surface answering "what does this app hold, from which gesture, and can I take it back?" — backed by the live authority graph ([[03_capability_model]]).
- **App identity & anti-spoofing.** Every grant prompt and picker authentically shows the requesting principal ([[27_windowing_and_compositor]], [[05_identity_and_principals]]); a hostile app cannot dress as a trusted one.
- **Background-task consent.** Authority for work the user is *not* watching (sync, location-in-background) should be a visible **lease**, renewed with consent, not a permanent grant.
- **Purpose at the gesture.** A grant can carry a declared purpose ([[08_data_model_and_privacy]]) so later use is checkable against *why* it was given.
- **Least-surprise defaults.** When ambiguous, mint the narrowest plausible capability and let the user widen explicitly — never the reverse.

## Key Questions

- What is the exact contract between a requesting app and the picker so the app receives only an object-scoped capability and never the surrounding namespace?
- How does drag-and-drop carry a capability across principal boundaries without a spoofable intermediate?
- What does the revocation surface show, and how does it map back onto delegation edges in the authority graph?
- How is background-task consent expressed as a lease the user can see and end?

## Omega Leverage

- Picker, drop, and share results are **capabilities + domains** minted by a trusted broker and **delegated** to the app — exactly Omega's broker/acquire pattern ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- Object-scoping is **attenuation** via a tighter [domain](../../../../Omega/wiki/language_guide/chapter_8_domains.md) (`Folder::Readable` → `File::Readable`).
- Background grants are **leases**, tied to trusted time and explicit renewal.
- The revocation surface reads the same authority-flow / graph data as [[03_capability_model]] — no separate bookkeeping.
- Omega does not define the *interaction* layer; the mint-on-gesture mapping is Cathedral's contribution, but it bottoms out in standard delegation.

## Open Questions

- Can the picker be made so reliable that category-wide grants ("all photos") are never needed — or are some workflows inherently broad?
- How much can we infer authority from gestures without a prompt before users feel the system is acting behind their back?

## Related
- [[03_capability_model]] — the held authority these gestures mint and move.
- [[04_capability_lifecycle]] — minting, delegation, attenuation, revocation.
- [[06_security_policy_and_sandboxing]] — ceilings that bound what a gesture can grant.
- [[27_windowing_and_compositor]] — the trusted path the picker must run on.
- [[08_data_model_and_privacy]] — purpose carried alongside the granted capability.
