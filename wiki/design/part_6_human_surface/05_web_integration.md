# Chapter 05: Web Integration

> Where Cathedral targets a browser-class device, the web is the largest existing app surface; the strong move is to make web *origins* into first-class principals in the capability system.

## The Legacy Model

Legacy OSes treat the browser as one opaque, maximally-trusted app and the web as a parallel universe with its own security model bolted alongside the OS's. The same-origin policy, web permissions, passkeys, service workers, and the storage sandbox are a sophisticated capability-ish system — but it is entirely *internal* to the browser, invisible to the OS, and duplicated badly when a web app is "installed." WebViews embedded in native apps re-expose every classic confused- deputy bug. The OS cannot reason about what a web origin can do, cannot grant it native authority safely, and cannot show the user one coherent picture of "what does this thing — native or web — actually hold?"

## The Cathedral Model

Unify the two models. **The strong move: a web origin becomes a principal in the OS capability system** — `https://app.example.com` holds and is granted capabilities exactly like a native app, native component, or device ([[identity_and_principals]], [[capability_model]]). The browser's same-origin boundary *is* a principal boundary the OS understands; a web permission *is* a capability grant routed through the same human gestures as native ones ([[human_permission_ux]]). This collapses the duplicated security model into one authority graph and makes "what can this origin do?" a normal query.

A concrete early decision: are web apps **first-class** (origins are full principals, installable, holding native capabilities under the same ceilings as native code via [[security_policy_and_sandboxing]]), **second-class** (web apps run but with a constrained capability menu), or **isolated legacy** (the browser is a sandboxed compatibility box, like other legacy runtimes in [[compatibility_and_legacy]])? The architecture differs sharply by answer; the design should commit to first-class as the aspiration and name the fallback.

### The decided direction (deferred — web is late-stage)

Web integration composes almost entirely from already-decided machinery, so the *shape* is settled even though the full mechanics are intentionally deferred: web comes late in the OS journey and is better revisited once the native stack exists.

**First-class, via nesting — born-low, promoted on install.** An origin is a **nested principal inside the browser-Matrix** (the recursive sub-principal pattern, [[identity_and_principals]]). By default a visited origin is **born-low, ephemeral, and near-empty** — a nested sub-world with full web reach but *no* native authority — which is cheap because nested Matrices are cheap (the open-web-client model, [[networking]]). On **install** (a deliberate user act) the origin is **promoted** into its own Matrix and may hold *delegated, attenuated* native capabilities granted through the OS gestures, like a native app. So principal-hood is universal, the default costs ~nothing, and weight is earned by promotion — resolving first-class-vs-second-class-vs-long-tail in one move.

**Identity = the origin's key; authentication = the first-pin problem (deferred).** An origin's principal id is its TLS/cert identity (legacy) or pinned key (native) — the same reachable-as-key identity as [[networking]] — so "how is an origin authenticated" *is* the first-introduction/first-pin problem parked in the security bucket ([network_trust_fabric](../../speculation/network_trust_fabric.md)). Nothing new; it inherits the deferred trust work.

**One authority graph → one revocation surface; web permissions are OS gestures.** Because origins are principals in the *same* graph, a web grant and a native grant are the same edge in the same Warden/legibility view, revoked the same way. Web permission prompts **become** the OS grant gestures (picker, action-confirm), so the browser's internal permission model is **replaced** for native browsing; the legacy web's own model survives only **contained inside the legacy browser** (native gets the clean model, legacy runs its old model in a box, [[compatibility_and_legacy]]). File-System-Access = the OS picker; passkeys = WebAuthn as an origin-tied **operation-capability** ([[secrets_and_keys]], use-not-read); all reuse.

**WebView confused-deputy dies by nesting.** An embedded WebView is recursive composition: the host hosts the web content as a *child Matrix* holding its **own** attenuated capabilities, never the host's, so host authority cannot ambiently leak in (identity does not propagate through intermediaries — [[identity_and_principals]]).

**Deferred residue:** origin authentication = first-pin (security bucket); the legacy web is vast and runs contained (near-zero *native* coverage at launch — the same adoption-gradient honesty as networking); WebGPU/WebCodecs bridge through device-service capabilities, but GPU is deferred. The full mechanics are intentionally left for a later pass.

## Concerns & Design Space

- **Origin as principal.** Map same-origin identity onto an OS principal; web permission prompts become capability grants in the unified graph.
- **Browser as app runtime / WebView security.** Embedded WebViews must not become confused deputies; each origin inside keeps its own principal identity.
- **Permissions mapping.** Web permission requests routed through the OS's human permission UX so native and web grants look and revoke the same ([[human_permission_ux]]).
- **Passkeys.** WebAuthn credentials as OS-managed authority tied to the origin principal ([[secrets_and_keys]]).
- **Filesystem access.** The web File System Access API expressed as the OS picker minting a narrow capability — same mint as native.
- **Web app installation & service workers.** Installed web apps as registered principals; background service workers as leased, consented background tasks.
- **Notifications & media DRM.** Routed through the same compositor/media surfaces ([[windowing_and_compositor]], [[media_and_graphics]]).
- **WebGPU / WebCodecs.** Bridged to native media capabilities and their device-service reach.
- **Sandboxing & native capability bridge.** A typed bridge so an origin gains native authority only by held capability, never ambiently.

## Key Questions

*(Direction resolved by "The decided direction" above, mechanics deferred: first-class **via nesting** (origin = a born-low nested principal in the browser-Matrix, promoted to its own Matrix on install); origin id = its key, auth = the deferred first-pin; one authority graph → one revocation surface, web permissions = OS gestures; WebView confused-deputy dies by nesting.)*

- Are web apps first-class, second-class, or isolated legacy — and what is the fallback if first-class proves too costly for the first device?
- What is the exact mapping from same-origin identity to an OS principal, and how is an origin authenticated (TLS identity, the resolver of [[naming_and_discovery]])?
- How do web permissions and native capabilities share one revocation surface so the user sees a single authority picture?
- How is a WebView prevented from becoming a confused deputy for its host app's authority?

## Omega Leverage

- An origin is a **principal** holding **capabilities + domains** — no new mechanism, just a new kind of node in the authority graph ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- The native bridge is a **boundary provider**; crossing it requires a held capability and shows up in the authority-flow report.
- Web messages use ordinary numbered schemas and selected wire codecs ([wire protocols](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)).
- Origin-scoped authority is **attenuation** via [domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- Omega does not model HTTP/TLS origin semantics; mapping origin identity onto a principal is Cathedral's bridge work atop the capability model.

## Open Questions

- Can the browser's internal permission model be *replaced* by the OS's, or must the two coexist and stay reconciled?
- Does making origins principals scale to the long tail of websites a user visits once, or is principal-hood reserved for installed/granted origins?

## Related
- [[identity_and_principals]] — origins as principals.
- [[capability_model]] — origins holding capabilities in the authority graph.
- [[security_policy_and_sandboxing]] — ceilings over web and native alike.
- [[human_permission_ux]] — web permissions mapped to the OS grant gestures.
- [[compatibility_and_legacy]] — the browser as an isolated legacy runtime fallback.
