# Chapter 00: Windowing & Compositor

> The surface where pixels meet people: the compositor owns window arrangement, input routing, and, above all, the *trusted path* that lets a human know which app they are actually talking to.

## The Legacy Model

X11 and its descendants treat the display server as a shared, mostly-trusting bus. Any client that can connect can typically snoop global input, read the clipboard, screenshot other windows, register global hotkeys, and draw anywhere. The original threat model assumed one cooperating user on one trusted host. Wayland tightened this, but the security-critical surfaces are still negotiated per-compositor, ad hoc: which client owns input focus, who may read the clipboard, whether a window can claim to be someone else's. Worst of all, the human has *no reliable signal* of which application owns the surface in front of them. Overlay windows, borderless popups, and fullscreen takeovers let a malicious app impersonate a password dialog, a system prompt, or another app's chrome. The legacy contract has no concept of a path the user can *trust*.

## The Cathedral Model

The compositor is a privileged broker, not a shared bus. It mediates input, clipboard, notifications, and window state as **capability-gated effects**, and it guarantees a **trusted path**: a region of the screen, and a set of interactions, that no untrusted app can draw over, spoof, or synthesize. Input events are *routed* capabilities: an app receives keystrokes because the compositor delegated focus to it for this surface, not because it asked the bus. App identity (see [[identity_and_principals]]) is something the compositor can *display authentically*, so "which app is this?" is always answerable by the human, never forgeable by the app.

Permission-*grant* gestures (the file picker, share sheets, "open with") are authority-mint surfaces and live in [[human_permission_ux]]; media decode, GPU, and the low-level display pipeline live in [[media_and_graphics]]. This chapter owns *window management and its trust surface* only.

## Layers and the background surface

The compositor arranges surfaces in named layers rather than one flat stack: a **background layer** at the very bottom, normal windows above it, chrome (panel, dock, status) above those, transient overlays such as notifications, and the OS-owned **trusted-path layer** on top, which nothing below can draw over. A layer is a placement class the compositor grants, so "render to the background" is a capability distinct from "render a normal window."

A background program is then an ordinary app pinned to the background layer. Anything that can render can be a wallpaper: a static image, an animation, a game, a 3D scene, a whole browser. It is also the most confined kind of app, holding a render surface and, only if granted, input, with no files or network by default, so a downloaded animated background cannot read your data or touch the trusted-path layer. Because frames are requested through the compositor and the surface is a budgeted component ([[scheduler_and_resources]]), the compositor gates it: a fully occluded background renders zero frames, and on battery or when idle it throttles or suspends, which is where most "live wallpaper" waste belongs. A faulted background is a supervised component, so the compositor shows a static fallback and the supervisor restarts it.

## The desktop is a set of roles

"The desktop" welds together three separable roles, each a capability or a registration:

- **The back surface.** Renders the wallpaper layer. Pure presentation.
- **The desktop authority.** Holds the capability to the Desktop location in the user realm ([[filesystem_as_database]]), so it knows which files and shortcuts exist and can show and launch them.
- **The drop target.** Registered to receive drag-and-drop over the desktop region. A drop delegates a capability to the dropped object to whoever holds this role, the same authority-mint principle as the picker ([[human_permission_ux]]).

One component may hold one role, several, or all. Two programs (a render-only wallpaper plus a separate shell that owns the file roles and draws icons on a transparent layer above it) keep the wallpaper free of any file authority. One program holding all three is the integrated case, a 3D file viewer that renders dropped files as objects in its own scene. A component holding only the back surface, with no drop target bound, exposes no file dropping at all, because a dropped object has no recipient. The roles can stay in separate programs whenever the icons are a layer floated over the wallpaper, and they must fuse into one program when the files are rendered inside the scene, because there the presentation and the file model are the same object.

## The OS chrome and custom shells

The OS chrome (the panel, the launcher, the stock home and file browser) is not a slot an app can replace. It is permanent, OS-owned, always resident, and **dormant-able**: a custom app can put it to sleep and take the foreground, but it is never unbound, so a live chrome is always one gesture away. Installing a fancy shell makes the OS chrome cede the foreground and suspend, occlusion-gated to zero frames while backed out, while staying resident.

Becoming the active chrome is a capability. A back-surface app holding the **chrome-dormancy** capability may put the OS chrome dormant and run as the primary shell, with whatever shell-tier authority it was also granted: launch apps, own the desktop roles, composite other apps' surfaces into its own scene, up to running a game as the entire desktop. The capability is the clean knob for the gradient. A back-surface app with no shell caps is a wallpaper; with shell caps it is a real shell under the OS chrome; with chrome-dormancy it is a full-immersive replacement. Each step is a held, visible, revocable grant, so "what may suppress my OS chrome" is an authority-graph query, and revoking the grant un-naps the chrome at once.

The chrome-dormancy capability carries the piece that makes a custom chrome feel native rather than skinned: while it is active, the **OS key short-press is delegated to the custom shell**. A tap of the home key opens the custom shell's own launcher instead of the stock one, which is exactly what someone implementing a real OS chrome expects. The compositor still owns the key and simply routes the short-press to whoever is the active chrome, the custom shell when the capability is active and the native chrome otherwise.

The OS reserves one gesture, forever, and it is the minimum needed for reliability: the **OS key long-press wakes the native chrome**, for when the custom thing is scuffed. No capability can take it, the custom shell never receives it, and it works regardless of configuration, so one physical key splits by gesture, the short-press to the active chrome and the long-press to the OS. Because the custom shell is a supervised component, a crash or hang is contained and the long-press reaches a live native chrome anyway. This is the live, no-reboot escape, distinct from firmware recovery, which is reserved for a broken OS core rather than a broken shell ([[boot_and_trust_chain]]).

All of this is safe only because the security model never lived in the chrome. A custom shell is an ordinary app holding shell-tier capabilities and nothing more, and two things stay OS-owned no matter what is installed: the capability core, and the trusted-path layer, so login, unlock, and grant prompts are OS-drawn over even a fullscreen custom shell and cannot be forged. The zero state is the stock desktop: with no chrome-dormancy capability held, the OS chrome is active and owns the OS key, so an unconfigured machine shows the native chrome over the default wallpaper, and immersion is something you opt into by granting the capability ([[omega_substrate]] ZII).

## Recursive composition

The compositor is a recursive interface any surface-holder can implement, rather than a single privileged process. Its operations are: mint child surfaces, receive their frames, route input down, and present the result upward. Any component that holds a surface and can spawn children can offer that interface to those children, so composition nests.

A component imports the `Compositor` protocol (the trait and its wire types), implements it, and at spawn time binds the child's compositor capability to its own endpoint. The child does the ordinary thing, resolving the compositor from its environment and submitting frames, and cannot tell whether the other end is the root compositor or a parent app. This is the synthetic-realm move ([[filesystem_as_database]]) applied to display: hand a child a capability to an interface and be the thing behind it. A nested compositor is often a thin proxy that holds a capability to its own real compositor and forwards what it does not intercept, so it overrides layout and input without reimplementing rendering.

Being a child's compositor grants display and input authority over that child: the parent sees every pixel the child draws, feeds it input, and can lie about size, focus, and visibility. It grants nothing else, because the child's file, network, and device capabilities are still its own from the real authority chain. The invariant that survives the nesting is the trusted path. Trust interactions escalate to the root, where the OS draws them over everyone, addressed to the principal by an identity it knows from the authority graph regardless of who composites it, so a nested compositor cannot phish the children it draws. Leases and gating recurse with the tree: when a parent dies the root tears down its subtree, and an occluded parent that stops pulling frames quiesces everything beneath it.

## One axis from window to virtual machine

Sub-compositing plus synthetic interfaces is what a virtual machine is, at the limit. Spawn a guest into an environment where the interfaces it resolves are bound to implementations you serve, and you are its world. The depth of that synthesis is a single axis:

- **Nested compositor.** Intercept display and input. The app believes it has a normal window system.
- **Synthetic environment.** Also bind its filesystem, network, and device interfaces. The app believes it has a whole machine. This is the container case from [[filesystem_as_database]], extended to the screen.
- **Full virtual machine.** Synthesize at the hardware level and run a foreign kernel.

The split is at the bottom. A Cathedral-native guest is intercepted at the Omega service interfaces, so it is cheap confinement plus implementations you serve. A foreign operating system that expects real hardware needs the hardware-isolation wall ([[kernel_architecture]]): CPU virtualization, instruction trapping, and synthetic or paravirtualized devices, where the guest's display device is a synthetic framebuffer that becomes a surface the host composites. The shape is identical at every depth: synthesize the environment the guest expects, bind it to interfaces you implement, and the guest's output becomes a surface in your window.

## Concerns & Design Space

- **Trusted path.** A user-recognizable, app-unforgeable indicator of which principal owns the focused surface, plus a reserved region/gesture the OS owns outright (a "secure attention" the compositor alone can render).
- **Anti-spoofing.** No app may draw chrome that impersonates system UI or another app's identity; fullscreen, overlays, and always-on-top are capability-gated and visibly attributed.
- **Input routing.** Keystrokes, pointer, touch, and IME events delivered only to the focused surface's principal; global input capture is a rare, explicit, audited capability (accessibility, screen readers).
- **Clipboard.** A mediated transfer channel, not ambient shared memory: a paste is a delegated, one-shot capability over the chosen payload, not a poll-able global buffer (overlaps the authority-transfer view in [[human_permission_ux]]).
- **Notifications.** Attributed to a principal, rate-limited, and spoof-resistant; a notification cannot claim another app's identity.
- **Multi-window state restoration.** Window geometry/session restored across restart and hot swap ([[updates_and_hot_swap]]) as versioned state, without reviving stale authority.
- **App identity display.** The compositor renders publisher/identity facts it *holds*, so the label is trustworthy even when the app is hostile.
- **Zero value.** A zero surface is the inert null-object ([[omega_substrate]] ZII): draws, frames, and input routed to it are accepted and discarded as no-ops, so a child handed a zeroed compositor capability composites against nothing rather than crashing, and an uninitialized surface field never faults the pipeline.

## Key Questions

- What is the minimal hardware/firmware assist (if any) for an unspoofable trusted-path indicator across the OS's first target devices?
- How is input focus modeled as a capability that the compositor mints and revokes per surface, and how does revocation interact with in-flight events?
- Is the clipboard a capability transfer or a mediated channel, and who sees the payload before the receiving app does?
- How does session/window restoration carry geometry forward without carrying forward authority a restarted app should re-acquire?
- How is a child's compositor capability bound at spawn, and how does a trusted-path interaction escalate to the root and address the right principal across arbitrary nesting depth?
- What is the minimum a foreign-OS guest needs synthesized (devices, framebuffer) versus a native guest intercepted at the service interfaces?
- The OS chrome is dormant-able rather than replaceable; against a hostile custom shell holding chrome-dormancy, what is the minimum the OS must keep non-dormant beyond the long-press escape and the trusted-path layer?

## Omega Leverage

- Input, clipboard, and notification access are **capabilities + domains** over surface/principal handles, audited through the same authority-flow report as everything else ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- The compositor is an explicit **boundary provider**: the audited edge between proved Omega code and the raw display/input hardware.
- Window/session lifecycle is a natural **machine with states** ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md), [states](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)): created → mapped → focused → occluded → suspended → destroyed.
- Restored window state is **versioned data** ([versioned data](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)).
- The compositor is a **trait** ([traits](../../../../Omega/wiki/language_guide/chapter_13_traits.md)) any component can implement, so nesting is one interface with many implementations; a child resolves its compositor from its environment ([modules & imports](../../../../Omega/wiki/language_guide/chapter_14_modules_imports_visibility.md)), and sub-compositing is binding that resolution to a parent endpoint.
- Surface placement classes (background, normal, chrome, overlay, trusted-path) are **capabilities + domains**; "render to the background layer" is a strictly different grant from a normal window.
- Omega does **not** yet describe an unspoofable trusted-path primitive; that is a compositor + firmware obligation Cathedral defines on top of the language.

## Open Questions

- Can trusted path be guaranteed purely in software on commodity GPUs, or does it need a dedicated overlay plane the OS reserves?
- Should the compositor be one component or a small federation (input router, surface manager, identity renderer) with capabilities flowing between them?
- How deep can recursive composition nest before input latency or frame scheduling degrades, and does the occlusion-and-power gating recursion need a hard depth bound?

## Related
- [[human_permission_ux]] — the grant gestures the compositor must host on a trusted path.
- [[media_and_graphics]] — the display pipeline and GPU beneath window management.
- [[identity_and_principals]] — the app identity the compositor displays authentically.
- [[capability_model]] — input/clipboard/notifications as held capabilities.
- [[filesystem_as_database]] — realms and the synthetic-root move that recursive composition mirrors; the Desktop location behind the desktop roles.
- [[kernel_architecture]] — the hardware-isolation wall a foreign-OS guest needs at the bottom of the synthesis spectrum.
- [[scheduler_and_resources]] — budgets and the occlusion/power gating for background and nested surfaces.
