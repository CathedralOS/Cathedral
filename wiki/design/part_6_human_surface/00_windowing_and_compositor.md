# Chapter 00: Windowing & Compositor

> The surface where pixels meet people: the compositor owns window arrangement, input routing, and, above all, the *trusted path* that lets a human know which app they are actually talking to.

## The Legacy Model

X11 and its descendants treat the display server as a shared, mostly-trusting bus. Any client that can connect can typically snoop global input, read the clipboard, screenshot other windows, register global hotkeys, and draw anywhere. The original threat model assumed one cooperating user on one trusted host. Wayland tightened this, but the security-critical surfaces are still negotiated per-compositor, ad hoc: which client owns input focus, who may read the clipboard, whether a window can claim to be someone else's. Worst of all, the human has *no reliable signal* of which application owns the surface in front of them. Overlay windows, borderless popups, and fullscreen takeovers let a malicious app impersonate a password dialog, a system prompt, or another app's chrome. The legacy contract has no concept of a path the user can *trust*.

## The Cathedral Model

The compositor is a privileged broker, not a shared bus. It mediates input, clipboard, notifications, and window state as **capability-gated effects**, and it guarantees a **trusted path**: a region of the screen, and a set of interactions, that no untrusted app can draw over, spoof, or synthesize. Input events are *routed* capabilities: an app receives keystrokes because the compositor delegated focus to it for this surface, not because it asked the bus. App identity (see [[identity_and_principals]]) is something the compositor can *display authentically*, so "which app is this?" is always answerable by the human, never forgeable by the app.

Permission-*grant* gestures (the file picker, share sheets, "open with") are authority-mint surfaces and live in [[human_permission_ux]]; media decode, GPU, and the low-level display pipeline live in [[media_and_graphics]]. This chapter owns *window management and its trust surface* only.

## Layers and system prompts

The compositor arranges surfaces in named layers rather than one flat stack: normal app windows, the chrome (panel, dock, status) above them, transient overlays such as notifications, and on top of everything the **system-prompt layer**: login, unlock, permission dialogs, secure text entry. This is the "trusted path" named plainly — the one part of the screen the OS draws itself. Nothing can draw over it, nothing can capture it, nothing can imitate it, and input directed at it goes straight to the OS without passing through any app stage. Every capability table in this chapter has a row that reads "system prompts: nobody"; this layer is that row.

The display powers, in one chart:

| Surface power | Who holds it | Notes |
|---|---|---|
| Draw into your own window | Every app | Comes with having a window; all an app ever owns |
| OS chrome (wallpaper included) | OS | Permanent, always resident, never replaceable |
| Custom chrome | Apps holding the be-the-shell grant | Hides the OS chrome, never disables it; OS key long-press always returns it |
| Host other apps' windows in your scene | Sandboxes, virtual desktops, custom chromes | The hosted app can't tell; its file/network authority is untouched |
| System prompts | OS only | Above everything, including a fullscreen custom chrome |

## OS chrome and custom chromes

Two chromes. No more modularity than that.

The **OS chrome** is the OS-owned shell: panel, launcher, the stock home and file browsing surface, and the **wallpaper** — static or animated — as a chrome feature fed by content, not an app slot. It is permanent, always resident, never replaceable, and it holds the capability to the Desktop location in the user realm ([[filesystem_as_database]]), so showing and launching what lives there is chrome functionality. There is no separately grantable "background layer" and no third-party wallpaper-app class: a live wallpaper is content the chrome renders, budgeted and occlusion-gated like everything else ([[scheduler_and_resources]]).

A **custom chrome** is one grant: *be the shell.* An app holding it puts the OS chrome to sleep and owns the screen — launcher, desktop, wallpaper, compositing other apps into its own scene, up to a game running as the entire desktop. The OS chrome is napping, never gone: it stays resident, occlusion-gated to zero frames, and the **OS key long-press always wakes it**, regardless of configuration, for when the custom chrome is scuffed. No capability can take that gesture and the custom chrome never receives it; one physical key splits by gesture, short-press to the active chrome (so a custom launcher opens like a native one), long-press to the OS. A crashed or hung custom chrome is a supervised component, so the long-press reaches a live OS chrome anyway. This is the live, no-reboot escape, distinct from firmware recovery ([[boot_and_trust_chain]]). The grant is held, visible, and revocable, so "what may hide my OS chrome" is an authority-graph query, and revoking it un-naps the chrome at once.

All of this is safe only because the security model never lived in the chrome. A custom chrome is an ordinary app holding shell-tier capabilities and nothing more, and two things stay OS-owned no matter what is installed: the capability core, and the system-prompt layer, so login, unlock, and grant prompts are OS-drawn over even a fullscreen custom chrome and cannot be forged. The zero state is the stock desktop: with no shell grant held, the OS chrome is active and owns the OS key ([[omega_substrate]] ZII).

(Open: drag-and-drop onto the desktop previously hung off a separate drop-target role; under the two-chrome model the desktop drop target is chrome functionality, and the cross-app drop contract — a drop delegates a capability to the dropped object, the same mint as the picker — keeps its home in [[human_permission_ux]].)

## Recursive composition

The compositor is a recursive interface any surface-holder can implement, rather than a single privileged process. Its operations are: mint child surfaces, receive their frames, route input down, and present the result upward. Any component that holds a surface and can spawn children can offer that interface to those children, so composition nests.

A component imports the `Compositor` protocol (the trait and its wire types), implements it, and at spawn time binds the child's compositor capability to its own endpoint. The child does the ordinary thing, resolving the compositor from its environment and submitting frames, and cannot tell whether the other end is the root compositor or a parent app. This is the synthetic-realm move ([[filesystem_as_database]]) applied to display: hand a child a capability to an interface and be the thing behind it. A nested compositor is often a thin proxy that holds a capability to its own real compositor and forwards what it does not intercept, so it overrides layout and input without reimplementing rendering.

Being a child's compositor grants display and input authority over that child: the parent sees every pixel the child draws, feeds it input, and can lie about size, focus, and visibility. It grants nothing else, because the child's file, network, and device capabilities are still its own from the real authority chain. This is also the system's observation mechanism stated from the other side: a host observes what it hosts *by construction*, because its children's output literally arrives in its hands. A dev harness or test runner observes the app it spawned with no grant at all — it is that app's compositor — and by the same token no privacy flag protects a surface from its own host. Topology beats flags; protection is always against capture grants and siblings, never against the thing compositing you, which is why system prompts are safe (the OS root hosts them) and an app inside a hostile host is not. The invariant that survives the nesting is the trusted path. Trust interactions escalate to the root, where the OS draws them over everyone, addressed to the principal by an identity it knows from the authority graph regardless of who composites it, so a nested compositor cannot phish the children it draws. Leases and gating recurse with the tree: when a parent dies the root tears down its subtree, and an occluded parent that stops pulling frames quiesces everything beneath it.

## One axis from window to virtual machine

Sub-compositing plus synthetic interfaces is what a virtual machine is, at the limit. Spawn a guest into an environment where the interfaces it resolves are bound to implementations you serve, and you are its world. The depth of that synthesis is a single axis:

- **Nested compositor.** Intercept display and input. The app believes it has a normal window system.
- **Synthetic environment.** Also bind its filesystem, network, and device interfaces. The app believes it has a whole machine. This is the container case from [[filesystem_as_database]], extended to the screen.
- **Full virtual machine.** Synthesize at the hardware level and run a foreign kernel.

The split is at the bottom. A Cathedral-native guest is intercepted at the Omega service interfaces, so it is cheap confinement plus implementations you serve. A foreign operating system that expects real hardware needs the hardware-isolation wall ([[kernel_architecture]]): CPU virtualization, instruction trapping, and synthetic or paravirtualized devices, where the guest's display device is a synthetic framebuffer that becomes a surface the host composites. The shape is identical at every depth: synthesize the environment the guest expects, bind it to interfaces you implement, and the guest's output becomes a surface in your window.

## Input

Input has two layers that stay strictly separated. At the bottom, an input
device driver ([[driver_model]]) holds the transport capability and translates
raw reports into a normalized numbered protocol event vocabulary (key, relative
pointer, absolute pointer, axis, button, touch, pressure, and so on). Every
device quirk is absorbed there. The compositor consumes only typed events and
never sees the raw device.

This standardizes at the event-vocabulary level even though devices do not standardize at the hardware level. HID (Human Interface Device) is a genuine self-describing bottom standard: a device declares its axes, buttons, and usages, and most input hardware speaks it, which makes input far more tractable than the GPU situation. The slop is the long tail (broken descriptors, vendor gesture processing, proprietary protocols, devices that lie), and it is contained in per-device drivers and quirk tables rather than leaking upward. A genuinely novel device ships a driver that maps its raw input to typed events, extending the vocabulary with a new kind. The normalized vocabulary follows the capability, profile, and action layering below, though the full set of capability kinds and the registry that governs them remain to be specified.

Routing is capability work. **Focus is a permission**: a per-surface capability the compositor mints and revokes, so a surface receives input because focus was delegated to it, free by virtue of being focused and gone the moment it is not. Focus also cannot be stolen: an app may request focus or self-raise, and the request is silently deniable — it no-ops with a success shape, the ZII answer to a rude question — with OS cooldowns on repeat offenders. Within a hosted subtree the host assigns focus freely among its children, because routing focus inside your own subtree is what being a host *is*; automation that needs to drive focus therefore hosts the things it automates rather than stealing from siblings.

Global capture is a separate, rare, audited capability (screen readers, last-resort assistive input) that sees input regardless of focus. Under nesting, "global" attenuates per level: a nested compositor can grant global-within-its-own-subtree but not the real global, because it only holds the input its parent routed to it — "global" always means global within the host that accepted the registration, whether that host is the real root or a sandbox three dreams deep. Global *hotkeys* are the narrow cousin: an app registers specific chords with its host's router — never the key stream — and the registration is a visible permission surfaced at install or launch. (Open: exact registration and conflict-arbitration semantics when two apps want the same chord.)

Above the driver, input is a pipeline of typed-event transformers from raw events up to the focused app: gesture recognition, input-method composition, accessibility remapping, and the like. Each stage is a component, and the powerful ones (global capture, an input method, a remapper, a nested compositor) are capabilities that insert a stage, so "global input" is just a high position in the pipeline, and an app takes raw or cooked input by which capability it holds. The OS ships a trusted key remapper as a stock stage, extendable or replaceable the way the chrome is: a third-party remapper is a granted stage, and system-prompt input bypasses every stage regardless.

Two pressures shape the pipeline. Latency: the cooked stages cost *meaning-time*, not compute — a gesture recognizer must wait to know tap from drag-start, an input method deliberately holds keys while composing, and the compositor coalesces high-rate samples to frame boundaries for apps that only repaint per frame. So high-rate devices (8 kHz pointers, pens) and games get a short-circuit capability: raw input skips the meaning-stages and the coalescing and nothing else — events still flow through the router, so focus enforcement and revocation hold. Raw is unprocessed, never unaccountable. Security: input is keylogging-sensitive, so any stage that sees keystrokes is significant audited authority and "who can see my keystrokes" is an authority-graph query. System-prompt input (login, unlock, grant prompts) bypasses every app stage and goes straight to the OS, so no input method or capture grant ever observes a credential.

The vocabulary is layered so device specifics push down and meaning pushes up, and the OS core never enshrines "gamepad" or "VR controller" as a concept. Those are data, not core taxonomy.

- **Capabilities (self-describing).** A device declares what it physically has
  in an extensible numbered protocol registry: buttons, typed axes, touch
  contacts, poses, pressure, and haptic outputs. Stable member identities and
  reader-tolerant codecs make compatible additions possible without a special
  versioned type ([[driver_model]]).
- **Profiles (semantic classes).** Recognizing a set of capabilities as a standard gamepad or a particular extended-reality controller is shared, content-addressed, updatable data, the equivalent of the SDL gamepad database or OpenXR interaction profiles. A new controller gets a stable named layout by shipping profile data rather than an OS update, so profiles stay convention rather than core.
- **Actions (the app interface that does not age out).** An app declares abstract actions ("primary fire", "move 2D", "grab pose", "menu") and a remappable binding maps device capabilities onto them. This is OpenXR's action system: a new device works without the app changing because the binding adapts, and accessibility and remapping plug into the same indirection. The binding is a higher stage in the pipeline above.
- **The raw escape hatch.** The unprofiled capability stream is always available by capability, so a device nobody has profiled yet is never blocked. An app picks its layer: raw, profile, or actions.

The costs are honest. The capability registry needs an open but curated namespace ([[naming_and_discovery]]); profiles are perpetual maintained data; the action indirection adds latency, so latency-critical apps may read capabilities directly; and predicting a 6-degree-of-freedom pose to a future display time is a real-time problem the clock substrate carries ([[time_and_clocks]]). The payoff is that the core stays a thin typed-event router, and every future device is a new capability kind plus a profile plus bindings rather than a change to the system.

## Cursor

The cursor is the compositor's, outright: position, sprite, visibility. Apps never draw the real one. What apps get is decomposed:

- **Shape is a request.** An app asks for an I-beam or a hand over its own surface, honored while the pointer hovers it. A courtesy, not a power.
- **Position is private.** Pointer events arrive at the hovered or focused surface in that surface's own local coordinates. "Where is the mouse, globally, right now" — which legacy OSes hand to every process ambiently, leaking what the user is doing machine-wide — is an observation grant like any other.
- **Capture is a grant.** Hide-confine-relative (pointer lock for games and 3D) is per-surface, with a reserved OS gesture that always frees the pointer — the cursor's long-press.
- **Warping is bounded.** An app may reposition the pointer inside its own capture; outside its own surface, never. Warp-to-misdirect dies as a class.
- **Agents are embodied.** Agent-driven input gets its **own cursor**, visually distinct, rendered and labeled by the compositor with the acting principal's identity. The human keeps their pointer, watches the agent work in real time, and can interrupt at any moment; "who is doing this" is answered by the trusted renderer, not by the app or the agent. Multi-cursor also quietly covers remote collaboration. (Open: whether a named dispatch animates the agent cursor to its target — likely yes, so every act is visible — and whether per-cursor event streams stay separate or merge for apps that don't care.)

A fake cursor drawn inside an app's own surface only fools clicks on that app. It cannot extend over the system-prompt layer, and the real cursor over a grant prompt is OS-rendered, so the offset-cursor consent attack needs two things it cannot get.

**Seats: "the user" is whoever holds the input.** A **seat** — an input routing (cursor + key focus) bound to a principal — is the first-class primitive, here from the start (multi-cursor done early rather than bolted on late). Physical devices feed seats, and so does virtual input, **indistinguishable to the consuming surface** (drive, test, record, or replay a world by feeding it a seat — the input arm of the synthetic-world model) yet **attributed to the observer** (each seat labeled by its principal). The OS alone knows which seat the *physical* devices feed, so a human-only gate is "input from the **OS-attested physical seat**," verifiable by the OS even though no app can tell. Agents are seated principals (a labeled cursor is a virtual seat), so an agent driving is *concurrent and visible*, not a hidden takeover — and the **OS-key escape acts on the physical-device→seat binding**, snapping real input back to the operator's seat, the one input a software agent cannot supply ([[identity_and_principals]], [[agents_as_principals]]).

## Legibility and observation

A surface is frames plus, optionally, **legibility annotations**: structured descriptions of what the frames mean. Comprehension and action are different problems with different channels — text and structure for *reading* (assistive tech, agents, dev tools), named dispatch for *acting* — and the tiers compose, with zero annotations a valid surface ([[omega_substrate]] ZII):

- **Pixels (the floor).** Every surface works through pixel interpretation — a local agent with OCR and bound detection that reads the screen and synthesizes actions. This is the only tier with universal coverage: a ported legacy app brings its own rendering stack down to the last glyph and will never emit anything, so the agent is the *contract* and structure is the *optimization*. The floor has to be genuinely good, not a degraded afterthought.
- **Text runs.** Published by the text stack as a byproduct of rendering ([[media_and_graphics]]): what text exists, in which surface, in what order. Covers reading, which is most of the problem, and cannot rot because the annotation comes from the same draw that produced the pixels.
- **Semantic nodes.** A toolkit-derived tree — role, label, value, state, declared actions and named hit-targets, intra-app focus — for apps that want full interaction support. Intra-app focus *logic* stays the app's concern; the tree only reports its answer.

Annotations are **retained state, not a per-frame stream**. A run or node is announced once, updated when its content changes, removed when it dies; a 60 fps game with a swinging camera emits nothing until a string changes, so steady-state legibility costs zero. Geometry is **pulled, never pushed**: "where is this run right now" is a query the emitter answers from the transforms it holds at that moment — per-cluster placements when asked, a coarse quad with an approximate flag when that is all it honestly knows ([[media_and_graphics]]). The cost of *not* having this contract is measured, not guessed: proving a minimal tic-tac-toe canary's layout correct without platform support took a sibling project thousands of lines of bespoke headless harness, down to counting dark pixels to locate its own grid lines.

Observation follows the hosting tree. A host observes what it hosts by construction (see recursive composition above); everyone else needs a grant from one family, parameterized by **(subtree, channels, system-prompt inclusion)** — channels being pixels, text runs, nodes, and input events. Screen capture and read-structure are the same grant shape at different parameters, and attenuation is topology: a sandboxed Cathedral can grant "read everything" internally and it is still bounded at its own root. System prompts are included only for the audited assistive tier ([[human_permission_ux]]), because a reader must speak the grant prompt — as the attested string the OS rendered, not a pixel guess — while agents' grants exclude them by default: the human approves grants, not the agent reading over their shoulder. Secure text entry emits nothing to anyone ([[media_and_graphics]]).

Acting is click-equivalent or it is not this system. Dispatch is a routed click addressed in the target surface's own local coordinates — there is no global coordinate space; each hosting level applies the same parent-to-child transform real input crosses. If a dialog covers the button, the dispatched click hits the dialog, exactly as a finger would: ghost clicks are impossible by construction and the app's own guards run unchanged. Where a surface opted into the structure tier, dispatch can address a node id instead of a coordinate — the same stable-handle idea as an HTML element id or a UIA AutomationId: the id resolves to the node's *current* region at delivery time, so an agent never clicks coordinates that went stale between looking and acting. Without the structure tier the name layer simply does not exist, and dispatch is routed clicks at coordinates, like a finger. A **dry-run query** — "what would a click here land on?" — answers at the *surface* level only: which window receives it, or what covers it, because that is what routing owns. What the click means *inside* the app — which control, enabled or not — is the app's input handler, answerable only through the structure tier or by being the app. Anything that wants to drive an app *without* the screen is an ordinary service API with ordinary capabilities ([[ipc_and_service_invocation]]); the hybrid — RPC dressed as clicking, bypassing the interaction guards a developer built visually — cannot exist because dispatch has no non-routed form.

Emission is voluntary, forever. The default text stack emits out of the box, so most native apps are legible without choosing anything; custom stacks emit if they care; ported legacy code never will, and the pixel agent carries them. Coverage is incentive-shaped — the sharpest incentive being that a legible app is one your own coding agent can drive and test ([[testing_and_simulation]]) — and org or store policy can demand tiers where buyers care ([[multi_user_and_org_control]]), with the OS core staying neutral.

(Open: whether read-structure is viewport-scoped by default — a full model tree can expose content never rendered, the list items scrolled out of view, which is strictly *stronger* than capture over the same subtree; full-model access may be a separate step up.)

## Capture and per-stream compositing

Reading pixels is a grant over a node of the hosting tree: your own windows free (they are your output), one window via the share picker — picking it *is* the grant ([[human_permission_ux]]) — a subtree or the whole screen via an explicit audited grant, and system prompts never, for anyone: capture composites a hole where the password dialog is.

The compositor composites **per audience**. A surface can declare that capture streams receive different content than the display: absent (a labeled placeholder box), or substituted with app-supplied alternate content — the streamer sees their username, the stream sees a styled blocker, remote desktop gets the redacted composite. Enforcement happens at composite time, never in the consumer: the capture holder is never in possession of the real bytes, so there is no redaction step a hostile capture tool can skip. The placeholder is visibly a placeholder — viewers knowing something is hidden is honest. This is self-protection with the incentive pointed the right way (only the app that wants its field blanked does anything, for itself), and it is the same mechanism that blanks system prompts, generalized. What it cannot hide is metadata: a substituted region still has a size, a position, and a change cadence, and that residue is acknowledged rather than solved.

Two symmetric facts close the loop. Every active observation grant — capture, structure, input, injection — has an **OS-chrome-drawn indicator** that no chrome, custom included, can hide: being watched is always visible. And the inverse is a free query: an app can ask whether anything currently holds an observation grant over it. The query's honest scope is the world your host shows you — it is answered by your compositor, and a hosting compositor can lie about anything, including this, while inherently seeing you regardless. So the real anti-observation guarantee is two-part: verify the hosting chain is the genuine root (an attestation question, [[boot_and_trust_chain]]), and then the query is truthful. A game or banking app under an attested root can *verify* it is unobserved — stronger than any legacy OS, where any process screenshots anything silently; inside an unattested sandbox the answer is advisory at best.

## Modalities

Some states gate whole classes of grants at once rather than revoking anything. A **modality** is a master valve: grants persist, but the router and compositor refuse to exercise classes of them while the modality holds, and leaving it restores everything instantly because nothing was destroyed.

- **Locked.** Nothing is exercisable: capture sees nothing, dispatch is refused, hotkeys are silent, the agent cursor freezes. The lock screen is a different world, not a filtered view of this one.
- **Chrome-summoned.** The OS-key long-press is a milder valve while the native chrome is being recalled. (Open: exactly which classes it suspends — injection and capture at minimum.)
- **Secure entry.** A password field is a micro-modality for its duration: capture and ordinary observers are blanked for the focused field, input routes past every app stage, annotations do not exist. One deliberate carve-out: the audited assistive tier retains a defined minimum — masked echo, focus position, never content — because a blind user must be able to type a password, and accessibility must not die exactly where users need it most.

(Open: the general policy when an app's self-protection — secure entry, no-capture, input shielding — collides with an observation grant the user themselves approved. Passwords resolve for the app; an anti-macro game likely does too; the per-class table is unwritten.)

## Concerns & Design Space

- **Trusted path.** A user-recognizable, app-unforgeable indicator of which principal owns the focused surface, plus a reserved region/gesture the OS owns outright (a "secure attention" the compositor alone can render).
- **Legibility annotations.** Text runs and semantic nodes as optional, schema-stable channels alongside frames; the compositor brokers, never interprets; the pixel agent is the universal floor and assistive tech consumes the best tier per surface.
- **Anti-spoofing.** No app may draw chrome that impersonates system UI or another app's identity; fullscreen, overlays, and always-on-top are capability-gated and visibly attributed.
- **Input routing.** Keystrokes, pointer, touch, and IME events delivered only to the focused surface's principal; global input capture is a rare, explicit, audited capability (accessibility, screen readers).
- **Input normalization.** Drivers translate raw device reports (HID and the long tail of quirks) into a typed event vocabulary, so the compositor and apps see device-agnostic events ([[driver_model]]). The exact vocabulary and the driver-versus-higher-stage processing split are to be specified.
- **Input as a staged pipeline.** Gesture recognition, input methods, and accessibility remapping are capability-held stages between the normalizing driver and the focused app; raw or exclusive low-latency input is a short-circuit capability past the cooked stages.
- **Clipboard.** The gesture mints the grant: Ctrl+V is the user pushing this one payload to this one app, a delegated one-shot capability, so a "clipboard read" permission never exists and there is no poll-able global buffer (the picker principle, [[human_permission_ux]]). Clipboard history is OS-chrome functionality; a clipboard *manager* that watches every copy is a rare audited observation grant, same family as input capture — on legacy OSes it is a silent keylogger.
- **Notifications.** Attributed to a principal, rate-limited, and spoof-resistant; a notification cannot claim another app's identity.
- **Multi-window state restoration.** Window geometry/session restored across restart and hot swap ([[updates_and_hot_swap]]) as versioned state, without reviving stale authority.
- **App identity display.** The compositor renders publisher/identity facts it *holds*, so the label is trustworthy even when the app is hostile.
- **Zero value.** A zero surface is the inert null-object ([[omega_substrate]] ZII): draws, frames, and input routed to it are accepted and discarded as no-ops, so a child handed a zeroed compositor capability composites against nothing rather than crashing, and an uninitialized surface field never faults the pipeline.

## Key Questions

- What is the minimal hardware/firmware assist (if any) for an unspoofable trusted-path indicator across the OS's first target devices?
- How is input focus modeled as a capability that the compositor mints and revokes per surface, and how does revocation interact with in-flight events? *(Decided: a per-surface input queue; an event's destination is fixed at **enqueue** time, so a focus change only redirects *subsequent* events; the target's already-queued events **drain** (dropping them mid-stream would leave a key-down with no key-up). The window of risk is one event, and the security-critical path never enters app routing at all — secure-entry is a modality routing straight to the OS.)*
- How are clipboard payload formats negotiated at paste time, and what does the OS-chrome clipboard history expose, to whom?
- How does session/window restoration carry geometry forward without carrying forward authority a restarted app should re-acquire?
- How is a child's compositor capability bound at spawn, and how does a trusted-path interaction escalate to the root and address the right principal across arbitrary nesting depth?
- What is the minimum a foreign-OS guest needs synthesized (devices, framebuffer) versus a native guest intercepted at the service interfaces?
- The OS chrome is dormant-able rather than replaceable; against a hostile custom chrome holding the be-the-shell grant, what is the minimum the OS must keep non-dormant beyond the long-press escape and the system-prompt layer?
- Is read-semantics viewport-scoped by default (capture-equivalent), with full-model access a separate stronger grant — and where does offscreen content stop being "the screen" and start being "the app's data"?

## Omega Leverage

- Input, clipboard, and notification access are **capabilities + domains** over surface/principal handles, audited through the same authority-flow report as everything else ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- The compositor is an explicit **boundary provider**: the audited edge between proved Omega code and the raw display/input hardware.
- Window/session lifecycle is a natural **machine with states** ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md), [states](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)): created → mapped → focused → occluded → suspended → destroyed.
- Restored window state uses **explicit durable schemas and migrations**
  ([evolution and migration](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md)).
- The compositor is a **trait** ([traits](../../../../Omega/wiki/language_guide/chapter_14_traits.md)) any component can implement, so nesting is one interface with many implementations; a child resolves its compositor from its environment ([modules & imports](../../../../Omega/wiki/language_guide/chapter_15_modules_imports_visibility.md)), and sub-compositing is binding that resolution to a parent endpoint.
- Surface placement classes (normal, chrome, overlay, system prompts) are **capabilities + domains**; "be the shell" is a strictly different grant from owning a normal window.
- Legibility annotations use numbered protocol schemas and explicit codecs
  ([protocol schemas](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)),
  so the stable contract is the schema rather than a blessed library.
- Omega does **not** yet describe an unspoofable trusted-path primitive; that is a compositor + firmware obligation Cathedral defines on top of the language.

## Open Questions

- Can trusted path be guaranteed purely in software on commodity GPUs, or does it need a dedicated overlay plane the OS reserves?
- Should the compositor be one component or a small federation (input router, surface manager, identity renderer) with capabilities flowing between them?
- How deep can recursive composition nest before input latency or frame scheduling degrades, and does the occlusion-and-power gating recursion need a hard depth bound? *(Decided: no hard depth bound — by design nobody knows how deep they are (indistinguishability). Depth costs latency (a compose pass + input hop per level), so it is bounded by the scheduler's resource/CPU **budget** ([[scheduler_and_resources]]): a pathologically-nesting subtree blows its budget and is quiesced or killed like any runaway, not capped by a special depth limit.)*
- What is the normalized input event vocabulary, how extensible is it for novel device classes, and how much interpretation (gestures, composition) belongs in the driver versus shared higher stages?
- What latency does the staged input pipeline add, and when must a raw or exclusive short-circuit bypass the cooked stages for pointers, pens, and games?

## Related
- [[human_permission_ux]] — the grant gestures the compositor must host on a trusted path.
- [[media_and_graphics]] — the display pipeline and GPU beneath window management.
- [[identity_and_principals]] — the app identity the compositor displays authentically.
- [[capability_model]] — input/clipboard/notifications as held capabilities.
- [[filesystem_as_database]] — realms and the synthetic-root move that recursive composition mirrors; the Desktop location the OS chrome holds.
- [[kernel_architecture]] — the hardware-isolation wall a foreign-OS guest needs at the bottom of the synthesis spectrum.
- [[scheduler_and_resources]] — budgets and the occlusion/power gating for background and nested surfaces.
- [[driver_model]] — input device drivers normalize raw HID into the typed event vocabulary.
- [[audio]] — the shared low-latency concern for the input and audio pipelines.
