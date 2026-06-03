# Chapter 00: Windowing & Compositor

> The surface where pixels meet people: the compositor owns window arrangement, input routing, and — above all — the *trusted path* that lets a human know which app they are actually talking to.

## The Legacy Model

X11 and its descendants treat the display server as a shared, mostly-trusting bus. Any client that can connect can typically snoop global input, read the clipboard, screenshot other windows, register global hotkeys, and draw anywhere — the original threat model assumed one cooperating user on one trusted host. Wayland tightened this, but the security-critical surfaces are still negotiated per-compositor, ad hoc: which client owns input focus, who may read the clipboard, whether a window can claim to be someone else's. Worst of all, the human has *no reliable signal* of which application owns the surface in front of them. Overlay windows, borderless popups, and fullscreen takeovers let a malicious app impersonate a password dialog, a system prompt, or another app's chrome. The legacy contract has no concept of a path the user can *trust*.

## The Cathedral Model

The compositor is a privileged broker, not a shared bus. It mediates input, clipboard, notifications, and window state as **capability-gated effects**, and it guarantees a **trusted path**: a region of the screen, and a set of interactions, that no untrusted app can draw over, spoof, or synthesize. Input events are *routed* capabilities — an app receives keystrokes because the compositor delegated focus to it for this surface, not because it asked the bus. App identity (see [[identity_and_principals]]) is something the compositor can *display authentically*, so "which app is this?" is always answerable by the human, never forgeable by the app.

Permission-*grant* gestures (the file picker, share sheets, "open with") are authority-mint surfaces and live in [[human_permission_ux]]; media decode, GPU, and the low-level display pipeline live in [[media_and_graphics]]. This chapter owns *window management and its trust surface* only.

## Concerns & Design Space

- **Trusted path.** A user-recognizable, app-unforgeable indicator of which principal owns the focused surface, plus a reserved region/gesture the OS owns outright (a "secure attention" the compositor alone can render).
- **Anti-spoofing.** No app may draw chrome that impersonates system UI or another app's identity; fullscreen, overlays, and always-on-top are capability-gated and visibly attributed.
- **Input routing.** Keystrokes, pointer, touch, and IME events delivered only to the focused surface's principal; global input capture is a rare, explicit, audited capability (accessibility, screen readers).
- **Clipboard.** A mediated transfer channel, not ambient shared memory: a paste is a delegated, one-shot capability over the chosen payload, not a poll-able global buffer (overlaps the authority-transfer view in [[human_permission_ux]]).
- **Notifications.** Attributed to a principal, rate-limited, and spoof-resistant; a notification cannot claim another app's identity.
- **Multi-window state restoration.** Window geometry/session restored across restart and hot swap ([[updates_and_hot_swap]]) as versioned state, without reviving stale authority.
- **App identity display.** The compositor renders publisher/identity facts it *holds*, so the label is trustworthy even when the app is hostile.

## Key Questions

- What is the minimal hardware/firmware assist (if any) for an unspoofable trusted-path indicator across the OS's first target devices?
- How is input focus modeled as a capability that the compositor mints and revokes per surface, and how does revocation interact with in-flight events?
- Is the clipboard a capability transfer or a mediated channel — and who sees the payload before the receiving app does?
- How does session/window restoration carry geometry forward without carrying forward authority a restarted app should re-acquire?

## Omega Leverage

- Input, clipboard, and notification access are **capabilities + domains** over surface/principal handles, audited through the same authority-flow report as everything else ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- The compositor is an explicit **boundary provider**: the audited edge between proved Omega code and the raw display/input hardware.
- Window/session lifecycle is a natural **machine with states** ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md), [states](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)): created → mapped → focused → occluded → suspended → destroyed.
- Restored window state is **versioned data** ([versioned data](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)).
- Omega does **not** yet describe an unspoofable trusted-path primitive; that is a compositor + firmware obligation Cathedral defines on top of the language.

## Open Questions

- Can trusted path be guaranteed purely in software on commodity GPUs, or does it need a dedicated overlay plane the OS reserves?
- Should the compositor be one component or a small federation (input router, surface manager, identity renderer) with capabilities flowing between them?

## Related
- [[human_permission_ux]] — the grant gestures the compositor must host on a trusted path.
- [[media_and_graphics]] — the display pipeline and GPU beneath window management.
- [[identity_and_principals]] — the app identity the compositor displays authentically.
- [[capability_model]] — input/clipboard/notifications as held capabilities.
