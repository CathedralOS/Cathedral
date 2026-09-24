# Chapter 05: Web Integration

> Where Cathedral targets a browser-class device, the web is the largest existing app surface. The strong move is to make web origins into principals in the capability system.

## The Legacy Model

Legacy OSes treat the browser as one opaque, maximally trusted app, and the web as a parallel universe with its own security model bolted alongside the OS's. The same-origin policy, web permissions, passkeys, service workers, and the storage sandbox are a sophisticated capability-like system, but it is entirely internal to the browser, invisible to the OS, and duplicated badly when a web app is "installed". WebViews embedded in native apps re-expose every classic confused-deputy bug. The OS cannot reason about what a web origin can do, cannot grant it native authority safely, and cannot show the user one coherent picture of what a given thing, native or web, holds.

## The Cathedral Model

The two models are unified: a web origin is a principal in the OS capability system. `https://app.example.com` holds and is granted capabilities exactly like a native app, native component, or device ([[identity_and_principals]], [[capability_model]]). The browser's same-origin boundary is a principal boundary the OS understands, and a web permission is a capability grant routed through the same human gestures as native ones ([[human_permission_ux]]). This collapses the duplicated security model into one authority graph and makes "what can this origin do?" a normal query.

One early decision sets the architecture. Web apps can be **first-class** (origins are full principals, installable, holding native capabilities under the same ceilings as native code via [[security_policy_and_sandboxing]]), **second-class** (web apps run but with a constrained capability menu), or **isolated legacy** (the browser is a sandboxed compatibility box, like other legacy runtimes in [[compatibility_and_legacy]]). The architecture differs sharply by answer. The design commits to first-class as the aspiration and names the fallback.

### The decided direction (deferred: web is late-stage)

Web integration composes almost entirely from machinery designed elsewhere, so the shape is fixed even though the full mechanics are deferred. Web comes late in the OS journey and is better revisited once the native stack exists.

Origins are first-class via nesting: born low, promoted on install. An origin is a nested principal inside the browser Matrix (the recursive sub-principal pattern of [[identity_and_principals]]). By default a visited origin is **born-low**: ephemeral and near-empty, a nested sub-world with full web reach but no native authority. This is cheap because nested Matrices are cheap (the open-web-client model of [[networking]]). On install, which is a deliberate user act, the origin is **promoted** into its own Matrix and may hold delegated, attenuated native capabilities granted through the OS gestures, like a native app. Principal-hood is universal, the default costs almost nothing, and weight is earned by promotion. That answers first-class versus second-class versus the long tail in one move.

Identity is the origin's key, and authentication is the first-pin problem, which is deferred. An origin's principal id is its TLS or certificate identity (legacy) or its pinned key (native), the same reachable-as-key identity as in [[networking]]. "How is an origin authenticated" is therefore the first-introduction or first-pin problem that sits in the security bucket ([network_trust_fabric](../../speculation/network_trust_fabric.md)). Nothing is new here; it inherits the deferred trust work.

One authority graph gives one revocation surface, and web permissions are OS gestures. Because origins are principals in the same graph, a web grant and a native grant are the same edge in the same Warden and legibility view, revoked the same way. Web permission prompts become the OS grant gestures (picker, action-confirm), so the browser's internal permission model is replaced for native browsing. The legacy web's own model survives only contained inside the legacy browser: native gets the clean model, and legacy runs its old model in a box ([[compatibility_and_legacy]]). File System Access is the OS picker. Passkeys are WebAuthn as an origin-tied operation-capability ([[secrets_and_keys]], use-not-read). All of it is reuse.

The WebView confused deputy dies by nesting. An embedded WebView is recursive composition: the host hosts the web content as a child Matrix holding its own attenuated capabilities, never the host's, so host authority cannot leak in ambiently. Identity does not propagate through intermediaries ([[identity_and_principals]]).

The deferred residue is small and named. Origin authentication is first-pin, in the security bucket. The legacy web is vast and runs contained, with near-zero native coverage at launch, the same adoption gradient as networking. WebGPU and WebCodecs bridge through device-service capabilities, but GPU is deferred. The full mechanics are left for a later pass.

## Concerns & Design Space

- **Origin as principal.** Same-origin identity maps onto an OS principal, and web permission prompts become capability grants in the unified graph.
- **Browser as app runtime / WebView security.** Embedded WebViews must not become confused deputies. Each origin inside keeps its own principal identity.
- **Permissions mapping.** Web permission requests route through the OS's human permission UX so native and web grants look and revoke the same ([[human_permission_ux]]).
- **Passkeys.** WebAuthn credentials are OS-managed authority tied to the origin principal ([[secrets_and_keys]]).
- **Filesystem access.** The web File System Access API is the OS picker minting a narrow capability, the same mint as native.
- **Web app installation & service workers.** Installed web apps are registered principals. Background service workers are leased, consented background tasks.
- **Notifications & media DRM.** Routed through the same compositor and media surfaces ([[windowing_and_compositor]], [[media_and_graphics]]).
- **WebGPU / WebCodecs.** Bridged to native media capabilities and their device-service reach.
- **Sandboxing & native capability bridge.** A typed bridge so an origin gains native authority only by held capability, never ambiently.

## Key Questions

- Are web apps first-class, second-class, or isolated legacy, and what is the fallback if first-class proves too costly for the first device?
- What is the exact mapping from same-origin identity to an OS principal, and how is an origin authenticated (TLS identity, the resolver of [[naming_and_discovery]])?
- How do web permissions and native capabilities share one revocation surface so the user sees a single authority picture?
- How is a WebView prevented from becoming a confused deputy for its host app's authority?

## Omega Leverage

- An origin is a principal holding capabilities plus domains. No new mechanism, just a new kind of node in the authority graph ([capabilities chapter](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- The native bridge is a boundary provider. Crossing it requires a held capability and shows up in the authority-flow report.
- Web messages use ordinary numbered schemas and selected wire codecs ([wire protocols](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#wire-protocols)).
- Origin-scoped authority is attenuation via [domains](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_8_domains.md).
- Omega does not model HTTP or TLS origin semantics. Mapping origin identity onto a principal is Cathedral's bridge work atop the capability model.

## Open Questions

- Can the browser's internal permission model be replaced by the OS's, or must the two coexist and stay reconciled? The direction is replacement for native browsing with the legacy model contained in a box; the mechanics are deferred.
- Does making origins principals scale to the long tail of websites a user visits once, or is principal-hood reserved for installed and granted origins? The direction is born-low nested principals for everyone, promoted on install.

## Related
- [[identity_and_principals]] — origins as principals.
- [[capability_model]] — origins holding capabilities in the authority graph.
- [[security_policy_and_sandboxing]] — ceilings over web and native alike.
- [[human_permission_ux]] — web permissions mapped to the OS grant gestures.
- [[compatibility_and_legacy]] — the browser as an isolated legacy runtime fallback.
