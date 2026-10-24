# Chapter 00: Windowing & Compositor

The accepted [compositor and seat contract](../../spec/human_surface/compositor_and_seat.md)
settles replaceable desktop programs and protected outer recovery chrome.
[ADR 0002](../../decisions/0002-kernel-platform-distribution.md) keeps shipped
programs in `distribution/` and common authority mechanisms in `platform/`.
The broader trusted-prompt, capture and seat proposals below remain design work;
they are not all ratified by this partial contract.

The accepted [input foundation](../../spec/human_surface/input.md) separately owns
buffered delivery, representations, target registration and recursive mappings.
The accepted [focus contract](../../spec/human_surface/focus_routes.md) now owns
local selection and transitions. The [presentation contract](../../spec/human_surface/rendering.md)
owns placement, demand and scoped fullscreen; detailed view/rendering protocols
below remain design work. Its separation of execution from presentation supersedes
any automatic subtree suspension implied by occlusion or stopped frame requests.

> The compositor owns window arrangement, input routing, and the trusted path that lets a human know which app they are talking to.

## The Legacy Model

X11 and its descendants treat the display server as a shared, mostly-trusting bus. Any client that can connect can typically snoop global input, read the clipboard, screenshot other windows, register global hotkeys, and draw anywhere. The original threat model assumed one cooperating user on one trusted host. Wayland tightened this, but the security-critical surfaces are still negotiated per compositor, ad hoc: which client owns input focus, who may read the clipboard, whether a window can claim to be someone else's. Worst of all, the human has no reliable signal of which application owns the surface in front of them. Overlay windows, borderless popups, and fullscreen takeovers let a malicious app impersonate a password dialog, a system prompt, or another app's chrome. The legacy contract has no concept of a path the user can trust.

## The Cathedral Model

The compositor is a privileged broker, not a shared bus. It mediates input, clipboard, notifications, and window state as capability-gated effects, and it guarantees a **trusted path**: a region of the screen, and a set of interactions, that no untrusted app can draw over, spoof, or synthesize. Input events are routed capabilities. An app receives keystrokes because the compositor delegated focus to it for this surface, not because it asked the bus. App identity ([[identity_and_principals]]) is something the compositor can display authentically, so "which app is this?" is always answerable by the human and never forgeable by the app.

This chapter owns window management and its trust surface only. Permission-grant gestures (the file picker, share sheets, "open with") are authority-mint surfaces and live in [[human_permission_ux]]. Media decode, GPU, and the low-level display pipeline live in [[media_and_graphics]].

The [rendering and composition proposal](../../proposals/0000_rendering_and_composition.md)
owns the candidate scene/resource mechanism beneath this interface. It separates
host authority and layout from pixel storage and final target execution. It does
not change input, capture, protected-layer or distribution ownership rules.

## Layers and system prompts

The compositor arranges surfaces in named layers rather than one flat stack: normal app windows, the chrome (panel, dock, status) above them, transient overlays such as notifications, and on top of everything the **system-prompt layer**: login, unlock, permission dialogs, secure text entry. This layer is the trusted path, the one part of the screen the OS draws itself. Nothing can draw over it, nothing can capture it, nothing can imitate it, and input directed at it goes straight to the OS without passing through any app stage. In the table below it is the "system prompts" row.

The display powers, in one chart:

| Surface power | Who holds it | Notes |
|---|---|---|
| Draw into your own window | Every app | Comes with having a window; all an app ever owns |
| Everyday chrome and background | Authorized selected programs | Separate replaceable roles; system realm supplies defaults |
| Outer recovery chrome | Enclosing system | Protected OS-key long press restores it; not required inside nested matrices |
| Custom chrome | Authorized selected program | May hide recovery chrome, never disable the outer recovery gesture |
| Host other apps' windows in your scene | Sandboxes, virtual desktops, custom chromes | The hosted app can't tell; its file/network authority is untouched |
| System prompts | OS only | Above everything, including a fullscreen custom chrome |

## Default programs, overrides and recovery chrome

The system realm supplies default desktop programs. Users may select custom
chrome, background and other programs through authorized configuration. A
background is an independent role, not necessarily a feature of the chrome.
Override slots in standard locations are a possible representation; no slot
schema or lookup precedence has been accepted yet.

The outer system retains a protected recovery chrome. A custom desktop may hide
it, but cannot remove it or intercept the physical OS-key long press that brings
it back. That input and presentation route must remain outside the custom
chrome's failure scope, including when its matrix crashes. Recovery is not a
promise that a failed kernel or every failed hardware provider can be bypassed.
The exact controls and process residency strategy remain implementation work.

Inner matrices need not contain recovery chrome. A parent may supply a scoped
system-realm copy with default programs; each child can select its own overrides
within its authority. That copy does not grant physical-input authority or access
to the outer recovery route. The distribution's default programs remain
replaceable by a fork; platform mechanisms enforce the protected boundary rather
than prescribing a second complete desktop.

Drag-and-drop and permission-grant interfaces remain separate contracts under
[[human_permission_ux]]. The recovery decision does not freeze those protocols or
the broader secure-prompt layer proposed above.

## Recursive composition

The Rust [matrix boundary lab](../../../source-rs/distribution/desktop/README.md)
now runs the same local-world input/grant implementation at two levels. Its
child protocol exposes only the child's own extent, attenuated keys and local
input; parent position, sibling metadata and depth are absent. This implements
neither the complete recursive compositor below nor a full confined environment.
Rendering integration, delegated process admission and the kernel metadata/timing
audit remain outstanding.

The [scene-submission proposal](../../proposals/0000_rendering_and_composition/scene_submission.md)
explores forwarding authorized descriptions and resource references through this
host chain. Receiving a frame and presenting upward need not require copying an
image at each ancestor. Delegation and host interception for that route are not
yet specified or implemented by the Rust rendering fixture.

The compositor is a recursive interface any surface-holder can implement, rather than a single privileged process. Its operations are: mint child surfaces, receive their frames, route input down, and present the result upward. Any component that holds a surface and can spawn children can offer that interface to those children, so composition nests.

A component imports the `Compositor` protocol (the boundary trait and its ordinary numbered external schemas), implements it, and at spawn time binds the child's compositor capability to its own endpoint. The child does the ordinary thing: it resolves the compositor from its environment and submits frames, and it cannot tell whether the other end is the root compositor or a parent app. This is the synthetic-realm move ([[filesystem_as_database]]) applied to display: hand a child a capability to an interface and be the thing behind it. A nested compositor is often a thin proxy that holds a capability to its own real compositor and forwards what it does not intercept, so it overrides layout and input without reimplementing rendering.

Being a child's compositor grants display and input authority over that child. The parent sees every pixel the child draws, feeds it input, and can lie about size, focus, and visibility. It grants nothing else, because the child's file, network, and device capabilities are still its own from the real authority chain.

This is also the system's observation mechanism stated from the other side. A host observes what it hosts by construction, because its children's output arrives in its hands. A dev harness or test runner observes the app it spawned with no grant at all, since it is that app's compositor, and by the same token no privacy flag protects a surface from its own host. Protection is always against capture grants and siblings, never against the thing compositing you. That is why system prompts are safe (the OS root hosts them) and an app inside a hostile host is not.

The invariant that survives the nesting is the trusted path. Trust interactions escalate to the root, where the OS draws them over everyone, addressed to the principal by an identity it knows from the authority graph regardless of who composites it, so a nested compositor cannot phish the children it draws. Leases and gating recurse with the tree: when a parent dies the root tears down its subtree, and an occluded parent that stops pulling frames quiesces everything beneath it.

There is no hard depth bound on nesting, because by design nobody knows how deep they are. Depth costs latency (a compose pass plus an input hop per level), so it is bounded by the scheduler's resource and CPU budget ([[scheduler_and_resources]]). A pathologically nesting subtree blows its budget and is quiesced or killed like any runaway, not capped by a special depth limit.

## One axis from window to virtual machine

Sub-compositing plus synthetic interfaces is what a virtual machine is, at the limit. Spawn a guest into an environment where the interfaces it resolves are bound to implementations you serve, and you are its world. The depth of that synthesis is a single axis:

- **Nested compositor.** Intercept display and input. The app believes it has a normal window system.
- **Synthetic environment.** Also bind its filesystem, network, and device interfaces. The app believes it has a whole machine. This is the container case from [[filesystem_as_database]], extended to the screen.
- **Full virtual machine.** Synthesize at the hardware level and run a foreign kernel.

The split is at the bottom. A Cathedral-native guest is intercepted at the Omega service interfaces, so it is cheap confinement plus implementations you serve. A foreign operating system that expects real hardware needs the hardware-isolation wall ([[kernel_architecture]]): CPU virtualization, instruction trapping, and synthetic or paravirtualized devices, where the guest's display device is a synthetic framebuffer that becomes a surface the host composites. The shape is identical at every depth: synthesize the environment the guest expects, bind it to interfaces you implement, and the guest's output becomes a surface in your window.

## Input

The [input and pointer proposal](../../proposals/0001_input_and_pointer_leases.md)
separates physical and virtual sources from logical pointers and their cursor
visuals. It explores Matrix-scoped leases and source multiplexing. The accepted
[focus contract](../../spec/human_surface/focus_routes.md) supersedes seat coupling;
cursor presentation and attribution remain design choices.

Input has two layers that stay separated. At the bottom, an input device driver ([[driver_model]]) holds the device's transport capability and translates raw device reports into a normalized, typed event vocabulary: ordinary numbered records under a selected codec for key, relative pointer, absolute pointer, axis, button, touch contact, pressure, and the like. Every device quirk is absorbed at that boundary. Above it, the compositor consumes only typed events and never sees the raw device, so routing is device-agnostic.

This standardizes at the event-vocabulary level even though devices do not standardize at the hardware level. HID (Human Interface Device) is a self-describing bottom standard: a device declares its axes, buttons, and usages, and most input hardware speaks it, which makes input far more tractable than the GPU situation. The slop is the long tail (broken descriptors, vendor gesture processing, proprietary protocols, devices that lie), and it is contained in per-device drivers and quirk tables rather than leaking upward. A novel device ships a driver that maps its raw input to typed events, extending the vocabulary with a new kind. The normalized vocabulary follows the capability, profile, and action layering below. The full set of capability kinds and the registry that governs them remain to be specified.

### Focus

The [accepted focus contract](../../spec/human_surface/focus_routes.md) separates
standing routing authority from local selection. A registered group has one
controlling owner, associates permitted channels and selects a direct child.
Keyboard and pointer handlers may request changes through that owner. Selection
alone grants no restore, raise or visibility authority.

Routes compose through matrices without exposing their complete ancestry.
Stopping upstream forwarding does not clear a child's local selection or make
its whole environment inactive. Independent sources can continue to operate.

A switch cancels affected router-owned events, orders cancellation to the old
recipient and suppresses held controls until release before they can activate a
new target. Published IPC events may still be consumed; there is no one-event
risk bound. Stream cleanup does not erase the child's remembered selection.
Pointer capture is separate and survives keyboard selection changes only while
its own grant, target and forwarding path remain valid.

### Global capture and hotkeys

Global capture is a separate, rare, audited capability (screen readers, last-resort assistive input) that sees input regardless of focus. Under nesting, "global" attenuates per level. A nested compositor can grant global-within-its-own-subtree but not the real global, because it only holds the input its parent routed to it. "Global" always means global within the host that accepted the registration, whether that host is the real root or a sandbox three levels deep. Global hotkeys are the narrow cousin: an app registers specific chords with its host's router, never the key stream, and the registration is a visible permission surfaced at install or launch. The exact registration and conflict-arbitration semantics when two apps want the same chord are not yet specified.

### The pipeline

Above the driver, input is a pipeline of typed-event transformers from raw events up to the focused app: gesture recognition, input-method composition, accessibility remapping, and the like. Each stage is a component. The powerful ones (global capture, an input method, a remapper, a nested compositor) are capabilities that insert a stage, so "global input" is a high position in the pipeline, and an app takes raw or cooked input according to which capability it holds. The OS ships a trusted key remapper as a stock stage, extendable or replaceable the way the chrome is. A third-party remapper is a granted stage, and system-prompt input bypasses every stage regardless.

Two pressures shape the pipeline. The first is latency. The cooked stages cost meaning-time, not compute: a gesture recognizer must wait to know tap from drag-start, an input method holds keys while composing, and the compositor coalesces high-rate samples to frame boundaries for apps that only repaint per frame. High-rate devices (8 kHz pointers, pens) and games therefore get a short-circuit capability. Raw input skips the meaning-stages and the coalescing and nothing else. Events still flow through the router, so focus enforcement and revocation hold, and raw input is unprocessed but never unaccountable. The second is security. Input is keylogging-sensitive, so any stage that sees keystrokes is significant audited authority, and "who can see my keystrokes" is an authority-graph query. System-prompt input (login, unlock, grant prompts) bypasses every app stage and goes straight to the OS, so no input method or capture grant ever observes a credential.

### The vocabulary

The vocabulary is layered so device specifics push down and meaning pushes up, and the OS core never enshrines "gamepad" or "VR controller" as a concept. Those are data rather than core taxonomy.

- **Capabilities (self-describing).** A device declares what it physically has in an extensible, versioned registry: buttons, axes (each typed with absolute or relative, linear or angular, range, and units), touch contacts, 6-degree-of-freedom poses, skeletal or hand poses, pressure, and haptic outputs. Input devices are bidirectional, so rumble and force feedback are held output capabilities. HID usages are the proven model, and using an ordinary numbered schema with a selected wire codec ([[driver_model]]) makes a new capability kind a compatible schema evolution. The device describes its hardware and is never forced into a category that does not fit.
- **Profiles (semantic classes).** Recognizing a set of capabilities as a standard gamepad or a particular extended-reality controller is shared, content-addressed, updatable data, the equivalent of the SDL gamepad database or OpenXR interaction profiles. A new controller gets a stable named layout by shipping profile data rather than an OS update, so profiles stay convention rather than core.
- **Actions (the app interface that does not age out).** An app declares abstract actions ("primary fire", "move 2D", "grab pose", "menu") and a remappable binding maps device capabilities onto them. This is OpenXR's action system: a new device works without the app changing because the binding adapts, and accessibility and remapping plug into the same indirection. The binding is a higher stage in the pipeline above.
- **The raw escape hatch.** The unprofiled capability stream is always available by capability, so a device nobody has profiled yet is never blocked. An app picks its layer: raw, profile, or actions.

The costs are real. The capability registry needs an open but curated namespace ([[naming_and_discovery]]). Profiles are perpetual maintained data. The action indirection adds latency, so latency-critical apps may read capabilities directly. Predicting a 6-degree-of-freedom pose to a future display time is a real-time problem the clock substrate carries ([[time_and_clocks]]). The payoff is that the core stays a thin typed-event router, and every future device is a new capability kind plus a profile plus bindings rather than a change to the system.

## Cursor

The accepted [cursor contract](../../spec/human_surface/rendering.md#cursor-presentations)
uses each pointer's hover/capture target independently of keyboard focus. Apps
can request a style or no cursor. Matrices resolve their internal target and
propagate the request under owner policy; target loss restores that policy.

Cursor programs contribute bounded surfaces rather than writing the whole output.
Hiding is separate from capture, confinement or relative-motion locking. Children
cannot hide unrelated pointers or protected outer UI. Mandatory agent attribution,
hidden-target indicators and precise pointer-lock APIs remain design choices.

### Seats

A conventional seat is a policy arrangement over channels, focus groups and
pointer visuals, not a mandatory primitive coupling a cursor to one keyboard.
The [focus contract](../../spec/human_surface/focus_routes.md) permits independent
groups and several pointers sharing selection. Apps receive scoped channels and
choose their own roles; a visual marker need not have any input authority.
Physical provenance and the outer recovery gesture remain separately enforced.
Trusted labels and recursive cursor presentation remain proposed design work.

## Legibility and observation

A surface is frames plus, optionally, **legibility annotations**: structured descriptions of what the frames mean. Comprehension and action are different problems with different channels. Text and structure serve reading (assistive tech, agents, dev tools), and named dispatch serves acting. The tiers compose, and zero annotations is a valid surface ([[omega_substrate]] ZII):

- **Pixels (the floor).** Every surface works through pixel interpretation: a local agent with OCR and bound detection that reads the screen and synthesizes actions. This is the only tier with universal coverage. A ported legacy app brings its own rendering stack down to the last glyph and will never emit anything, so the agent is the contract and structure is the optimization. The floor has to be good, not a degraded afterthought.
- **Text runs.** Published by the text stack as a byproduct of rendering ([[media_and_graphics]]): what text exists, in which surface, in what order. This covers reading, which is most of the problem, and it cannot rot because the annotation comes from the same draw that produced the pixels.
- **Semantic nodes.** A toolkit-derived tree of role, label, value, state, declared actions and named hit-targets, and intra-app focus, for apps that want full interaction support. Intra-app focus logic stays the app's concern; the tree only reports its answer.

Annotations are retained state, not a per-frame stream. A run or node is announced once, updated when its content changes, and removed when it dies. A 60 fps game with a swinging camera emits nothing until a string changes, so steady-state legibility costs zero. Geometry is pulled, never pushed. "Where is this run right now" is a query the emitter answers from the transforms it holds at that moment: per-cluster placements when asked, or a coarse quad with an approximate flag when that is all it knows ([[media_and_graphics]]). The cost of not having this contract is measured, not guessed. Proving a minimal tic-tac-toe canary's layout correct without platform support took a sibling project thousands of lines of bespoke headless harness, down to counting dark pixels to locate its own grid lines.

### Observation grants

Observation follows the hosting tree. A host observes what it hosts by construction (see recursive composition above). Everyone else needs a grant from one family, parameterized by subtree, channels, and system-prompt inclusion, where the channels are pixels, text runs, nodes, and input events. Screen capture and read-structure are the same grant shape at different parameters, and attenuation is topology: a sandboxed Cathedral can grant "read everything" internally and it is still bounded at its own root. System prompts are included only for the audited assistive tier ([[human_permission_ux]]), because a reader must speak the grant prompt as the attested string the OS rendered, not a pixel guess. Agents' grants exclude them by default: the human approves grants, not the agent reading over their shoulder. Secure text entry emits nothing to anyone ([[media_and_graphics]]).

### Dispatch

Coordinate input follows the latest coherently accepted hit-test layout, which
may lead displayed pixels. Each hosting level applies its local bounds and
transforms. Targets cannot silently change after routing; exact screenshot/frame
matching is not guaranteed, and apps validate actions against current state.

The accepted [input contract](../../spec/human_surface/input.md) also permits
advertised semantic operations and standing subscriptions. Stable interaction
identities avoid coordinate guessing but do not freeze availability or app state.
A strict captured-layout action or a dry-run query can be explored separately;
neither promises that the world stays unchanged until a later action arrives.

### Emission

Emission is voluntary and stays voluntary. The default text stack emits out of the box, so most native apps are legible without choosing anything. Custom stacks emit if they care. Ported legacy code never will, and the pixel agent carries them. Coverage is incentive-shaped, and the sharpest incentive is that a legible app is one your own coding agent can drive and test ([[testing_and_simulation]]). Org or store policy can demand tiers where buyers care ([[multi_user_and_org_control]]), with the OS core staying neutral.

Whether read-structure is viewport-scoped by default is open. A full model tree can expose content never rendered, such as list items scrolled out of view, which is stronger than capture over the same subtree, so full-model access may be a separate step up.

## Capture and per-stream compositing

The accepted [capture boundary](../../spec/human_surface/rendering.md#capture)
requires an explicit capability to served, appropriately masked output. It grants
no access to the unmasked original. Detailed mask/indicator policies below remain
design proposals; they do not change the accepted hit-testing or resource rules.

Reading pixels is a grant over a node of the hosting tree. Your own windows are free, since they are your output. One window comes via the share picker, where picking it is the grant ([[human_permission_ux]]). A subtree or the whole screen needs an audited grant. System prompts are never capturable, for anyone: capture composites a hole where the password dialog is.

The compositor composites per audience. A surface can declare that capture streams receive different content than the display: absent (a labeled placeholder box), or substituted with app-supplied alternate content. The streamer sees their username, the stream sees a styled blocker, and remote desktop gets the redacted composite. Enforcement happens at composite time, never in the consumer. The capture holder is never in possession of the real bytes, so there is no redaction step a hostile capture tool can skip. The placeholder is visibly a placeholder, so viewers know something is hidden. This is self-protection with the incentive pointed the right way, because only the app that wants its field blanked does anything, for itself, and it is the same mechanism that blanks system prompts, generalized. What it cannot hide is metadata: a substituted region still has a size, a position, and a change cadence. That residue is acknowledged rather than solved.

Two symmetric facts close the loop. Every active observation grant (capture, structure, input, injection) has an OS-chrome-drawn indicator that no chrome, custom included, can hide, so being watched is always visible. The inverse is a free query: an app can ask whether anything currently holds an observation grant over it. The query's scope is the world your host shows you. It is answered by your compositor, and a hosting compositor can lie about anything, including this, while inherently seeing you regardless. The real anti-observation guarantee is therefore two-part: verify the hosting chain is the real root (an attestation question, [[boot_and_trust_chain]]), and then the query is truthful. A game or banking app under an attested root can verify it is unobserved, which is stronger than any legacy OS, where any process screenshots anything silently. Inside an unattested sandbox the answer is advisory at best.

## Modalities

Some states gate whole classes of grants at once rather than revoking anything. A **modality** is a master valve: grants persist, but the router and compositor refuse to exercise classes of them while the modality holds, and leaving it restores everything instantly because nothing was destroyed.

- **Locked.** Nothing is exercisable: capture sees nothing, dispatch is refused, hotkeys are silent, the agent cursor freezes. The lock screen is a different world, not a filtered view of this one.
- **Chrome-summoned.** The OS-key long-press is a milder valve while the native chrome is being recalled. Exactly which classes it suspends is open; injection and capture are suspended at minimum.
- **Secure entry.** A password field is a micro-modality for its duration: capture and ordinary observers are blanked for the focused field, input routes past every app stage, and annotations do not exist. One carve-out: the audited assistive tier retains a defined minimum of masked echo and focus position, never content, because a blind user must be able to type a password, and accessibility must not die exactly where users need it most.

The general policy when an app's self-protection (secure entry, no-capture, input shielding) collides with an observation grant the user themselves approved is open. Passwords resolve for the app. An anti-macro game likely does too. The per-class table is unwritten.

## Concerns & Design Space

- **Trusted path.** A user-recognizable, app-unforgeable indicator of which principal owns the focused surface, plus a reserved region or gesture the OS owns outright (a "secure attention" the compositor alone can render).
- **Legibility annotations.** Text runs and semantic nodes as optional, schema-stable channels alongside frames. The compositor brokers and never interprets. The pixel agent is the universal floor, and assistive tech consumes the best tier per surface.
- **Anti-spoofing.** No app may draw chrome that impersonates system UI or another app's identity. Fullscreen, overlays, and always-on-top are capability-gated and visibly attributed.
- **Input routing.** Keystrokes, pointer, touch, and IME events are delivered only to the focused surface's principal. Global input capture is a rare, audited capability (accessibility, screen readers).
- **Input normalization.** Drivers translate raw device reports (HID and the long tail of quirks) into a typed event vocabulary, so the compositor and apps see device-agnostic events ([[driver_model]]). The exact vocabulary and the driver-versus-higher-stage processing split are to be specified.
- **Input as a staged pipeline.** Gesture recognition, input methods, and accessibility remapping are capability-held stages between the normalizing driver and the focused app. Raw or exclusive low-latency input is a short-circuit capability past the cooked stages.
- **Clipboard.** The gesture mints the grant. Ctrl+V is the user pushing this one payload to this one app, a delegated one-shot capability, so a "clipboard read" permission never exists and there is no poll-able global buffer (the picker principle, [[human_permission_ux]]). Clipboard history is OS-chrome functionality. A clipboard manager that watches every copy is a rare audited observation grant, the same family as input capture; on legacy OSes it is a silent keylogger.
- **Notifications.** Attributed to a principal, rate-limited, and spoof-resistant. A notification cannot claim another app's identity.
- **Multi-window state restoration.** Window geometry and session are restored across restart and hot swap ([[updates_and_hot_swap]]) as versioned state, without reviving stale authority.
- **App identity display.** The compositor renders publisher and identity facts it holds, so the label is trustworthy even when the app is hostile.
- **Zero value.** A zero surface is the inert null-object ([[omega_substrate]] ZII). Draws, frames, and input routed to it are accepted and discarded as no-ops, so a child handed a zeroed compositor capability composites against nothing rather than crashing, and an uninitialized surface field never faults the pipeline.

## Key Questions

- What is the minimal hardware/firmware assist (if any) for an unspoofable trusted-path indicator across the OS's first target devices?
- How are clipboard payload formats negotiated at paste time, and what does the OS-chrome clipboard history expose, to whom?
- How does session/window restoration carry geometry forward without carrying forward authority a restarted app should re-acquire?
- How is a child's compositor capability bound at spawn, and how does a trusted-path interaction escalate to the root and address the right principal across arbitrary nesting depth?
- What is the minimum a foreign-OS guest needs synthesized (devices, framebuffer) versus a native guest intercepted at the service interfaces?
- The OS chrome is dormant-able rather than replaceable; against a hostile custom chrome holding the be-the-shell grant, what is the minimum the OS must keep non-dormant beyond the long-press escape and the system-prompt layer?
- Is read-semantics viewport-scoped by default (capture-equivalent), with full-model access a separate stronger grant, and where does offscreen content stop being "the screen" and start being "the app's data"?

## Omega Leverage

- Input, clipboard, and notification access are capabilities plus domains over surface and principal handles, audited through the same authority-flow report as everything else ([capabilities chapter](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- The compositor is a boundary provider: the audited edge between proved Omega code and the raw display and input hardware.
- Window and session lifecycle is a natural machine with states ([machines](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_3_machines.md), [states](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_4_states_transitions.md)): created → mapped → focused → occluded → suspended → destroyed.
- Restored window state uses immutable historical schemas and checked conversions ([historical data](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#versioned-data)).
- The compositor is a trait ([traits](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md)) any component can implement, so nesting is one interface with many implementations. A child resolves its compositor from its environment ([modules & imports](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_15_modules_imports_visibility.md)), and sub-compositing is binding that resolution to a parent endpoint.
- Surface placement classes (normal, chrome, overlay, system prompts) are capabilities plus domains. "Be the shell" is a different grant from owning a normal window.
- Legibility annotations (text runs, semantic nodes) use ordinary numbered schemas and selected wire codecs ([wire protocols](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#wire-protocols)), so the contract that stays stable is a schema rather than a blessed library, and a new node kind is a compatible evolution.
- Omega does not yet describe an unspoofable trusted-path primitive. That is a compositor plus firmware obligation Cathedral defines on top of the language.

## Open Questions

- Can trusted path be guaranteed purely in software on commodity GPUs, or does it need a dedicated overlay plane the OS reserves?
- Should the compositor be one component or a small federation (input router, surface manager, identity renderer) with capabilities flowing between them?
- At what nesting depth does recursive composition degrade input latency or frame scheduling in practice, given that the scheduler's budget is the only bound?
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
