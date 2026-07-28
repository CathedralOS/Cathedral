# Chapter 02: Media & Graphics

> The low-level pipeline beneath the window manager: display server, GPU, decode, and audio — capability-gated, and deliberately kept narrow, because graphics stacks are historically a major source of OS complexity and instability.

## The Legacy Model

The media stack is the least-contained part of a legacy OS. GPU access means a giant, vendor-specific userspace driver mapped into the app with broad DMA reach; video decode and audio mixing run through sprawling frameworks (VA-API/V4L2/PulseAudio/PipeWire/DRM-KMS) with their own ad hoc permission models. Screen capture is frequently all-or-nothing. DRM/content protection drags in opaque, attestation-hungry blobs. The result is enormous attack surface, poor isolation between apps sharing the GPU, and a pipeline so complex that "clean-slate" OS projects routinely founder trying to reimplement it.

## The Cathedral Model

Treat every media facility as a **capability-gated device-service boundary** over an explicit pipeline, and keep the *first target deliberately narrow* — enough to play video, render UI, and mix audio on one class of device, not a universal graphics platform on day one. The display server / compositor pipeline here sits *beneath* the window management of [[windowing_and_compositor]]: this chapter owns the bytes-and-frames layer, that chapter owns windows and trust. GPU, decode, and screen capture are all held capabilities, never ambient: an app renders because it holds a rendering capability, captures the screen because it holds a (rare, attributed) capture capability.

The discipline is *say no early*. Pick one GPU path, one decode path, one audio graph for the initial target; gate each behind a capability; route everything through the driver model ([[driver_model]]) so the untrusted vendor blob is isolated rather than mapped into apps.

## Why the GPU is the deliberately minimized concern

GPUs are the least standardized hardware in the machine, which is the whole reason they sit at the edge of the design rather than the center. The pain has two separable sources, and only one of them is the OS's to solve.

The first is vendor opacity. A discrete GPU's command interface and instruction set are undocumented and change every hardware generation, and the driver is millions of lines of closed vendor code that includes a runtime compiler turning shader bytecode (SPIR-V, PTX) into the GPU's secret instruction set and then runs that untrusted shader code on a separate processor. Standardization exists at the API level (Vulkan, SPIR-V, Metal) but not at the hardware level, so the layer the OS must actually drive stays proprietary. This is the opposite of the small, known, capability-confined driver the driver model wants, and it is why a clean GPU driver is not something an OS project can simply sit down and write.

The second is genuine hardware hardness, opaque or not. A GPU is a bus-master DMA engine that reaches all of system memory without an IOMMU, its preemption is coarse so a long-running kernel can hog the device between submissions, context isolation between clients has historically leaked video memory, and virtualization (the synthetic-device path from [[driver_model]]) is the hardest nesting case there is and is vendor-locked where it exists at all.

So the stance is to treat the heavy GPU as a large opaque boundary provider behind the hardware-isolation wall, the application side of the two-walls model in [[kernel_architecture]]: confined by the IOMMU, by capabilities over which clients it serves and which memory it may touch, and by a scheduling budget, but not proven. It is the canonical big foreign blob the OS walls off rather than verifies, the same treatment a C++ binary gets. This also fits how GPU drivers are already built, mostly in userspace (the Mesa or vendor userspace stack) with a smaller kernel piece for command submission, memory management, and modesetting, which aligns with the user-mode-driver preference in [[driver_model]].

The useful separation is between the display controller and the render engine. Getting the compositor's framebuffer onto the screen is scanout and modesetting, a far simpler and more separable job than the 3D and compute engine, and it can be driven by a comparatively clean display-controller driver. A working composited desktop then needs only scanout plus CPU or software rendering, with the heavy GPU as an optional, confined accelerator that graphics, compute, and ML clients reach through a budget. That decouples "the system boots and shows a trustworthy UI" from "the full GPU stack works," which matters because the compositor and its trusted path must never depend on the messiest component in the machine.

What Cathedral cannot do here is worth stating plainly: it cannot prove the vendor's shader compiler, it cannot make coarse hardware preemption schedule fairly, and it cannot dissolve the opacity. A genuinely clean GPU answer is partly a hardware-control decision, targeting a documented or open GPU, which is a hardware-strategy question the design deliberately stays out of.

## Text and fonts

Text rendering, shaping, layout, and locale formatting are userspace libraries rather than OS services. Turning codepoints and a font into positioned glyphs, and glyphs into pixels, is pure computation with no capabilities and no I/O, so it belongs in a shared standard library that renders into the app's own surface. The OS owns only two thin touchpoints, both already covered: the compositor supplies DPI and scale and composites the finished surface without ever seeing a glyph ([[windowing_and_compositor]]), and input methods are a capability-held stage in the input pipeline.

Because shaping is where every string becomes glyphs, the text stack is also where legibility falls out for free. A surface may publish **text-run annotations**: small ordinary records under a selected wire codec — source string, visibility, run and ordering ids — *retained, not streamed*. A run is announced once, updated when its string changes, removed when it dies, so steady-state legibility costs nothing and a 60 fps scene with an unchanged HUD emits nothing per frame. Geometry is pulled, never pushed: "where is this run right now" is a query the renderer answers from the transforms it already holds, and the answer is what the draw call computed, not screen boxes. The general form of that answer is per-cluster placement — every glyph cluster with its own position and orientation, which is exactly the data any layout produces to draw at all, so text on an arc or along a path answers as naturally as a flat label. A run whose clusters share one plane collapses to the compact form, string plus a single transform of its line box: the common case, an optimization rather than the model. Only text whose final geometry never exists CPU-side (warped in a shader after submission) falls back to a coarse hull with an approximate flag, with the pixel tier supplying exact geometry if anyone needs it; for reading, the string and rough location were always the payload anyway. The OS owns the *schema*, not the library: the standard text crate emits by default as a byproduct of rendering, so the annotation cannot drift from the pixels it describes, and a custom stack (a game engine's own font pipeline, text projected onto a 3D surface) emits the same schema if it wants its text legible — the renderer already holds the string and the transform, so emission is cheap and voluntary. Secure text entry emits nothing: a masked field never produces a run, so the secret is absent from the annotation stream rather than guarded inside it. The compositor brokers annotations without interpreting them; who may read them, and the tiers above text runs, live in [[windowing_and_compositor]]. The record shape and emission mechanism here are sketch, not spec; the settled commitments are only: opt-in, one standardized schema, retained rather than streamed, geometry answered on request.

Fonts and locale data are content-addressed shared assets ([[filesystem_as_database]]), not a special subsystem. The system ships a core UI font plus a broad Unicode coverage floor (open-licensed, with large scripts and emoji as on-demand content-addressed packs), and the chrome pins its own fonts by hash so the trusted path always renders. An app declares the fonts it renders in by hash, so the system set is a dedup cache and a coverage floor rather than an ambient dependency: a font the system already holds is shared for free, one it does not is shipped by the app, and either way the result is reproducible. That is why shipping fonts does not reintroduce the implicit-host-dependency problem realms otherwise kill.

Installing a font adds it to the user's font collection in the user realm, discoverable by querying font-typed objects, never a write into the immutable system realm; changing the system set is a versioned update or an org policy ([[multi_user_and_org_control]]) rather than a drop into a global folder. And because the text library is pure and confined, the historical font-parser remote-code-execution vector shrinks to corrupting at most the one surface being drawn, with no escape and no reach into the system chrome.

## Concerns & Design Space

- **Display server / compositor pipeline.** The frame-composition and scanout layer beneath window management; the boundary to GPU scanout hardware.
- **GPU access.** Capability-gated command submission with isolation between principals sharing the device; no ambient DMA reach.
- **Display controller vs render engine.** Scanout and modesetting are a separable, comparatively clean driver; the 3D/compute GPU is the opaque blob. A composited desktop can run on scanout plus software rendering, with the heavy GPU as an optional confined accelerator, so the trusted display path never depends on the vendor GPU stack.
- **Video decode.** A capability over a decode-session provider, not a mapped library with the run of memory.
- **Audio graph.** A routed graph of capability-held nodes; mixing and capture are distinct, separately granted.
- **DRM / content protection.** Quarantine the attestation blob behind a narrow boundary; decide *whether* to support it at all in the first target.
- **Screen capture.** A capability ([[capability_model]]), rare, attributed, and visibly indicated — never silent.
- **Window isolation & input routing.** Defer to [[windowing_and_compositor]]; the pipeline must not leak one principal's frames or input to another.
- **Accessibility & color management.** First-class consumers of the pipeline; accessibility needs a privileged-but-audited read path.
- **Frame scheduling / GPU budget.** GPU time and frame deadlines are scheduled resources ([[scheduler_and_resources]]).
- **Low-latency media & power-aware rendering.** Latency and energy as explicit budgets, with rendering that backs off under power policy ([[power_management]]).
- **Zero value.** A zero decode or render session is the inert null-object ([[omega_substrate]] ZII): submitting buffers to it is accepted and produces no output, so a zeroed session is a valid no-op pipeline that drains cleanly rather than faulting, never mapping the vendor blob or touching real device memory.

## Key Questions

- What is the single narrow GPU + decode + audio target for the first device, and what is explicitly out of scope?
- How is the untrusted vendor GPU/decode driver isolated so a bug there does not become app or kernel compromise ([[driver_model]])?
- How is screen capture surfaced as a capability with an unspoofable in-use indicator?
- How do frame deadlines and GPU budget integrate with the general scheduler rather than living in a private real-time silo?

## Omega Leverage

- GPU/decode/capture/audio are **capabilities plus reach to their boundary services**, audited on the same axes as any other boundary ([capabilities & effects](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- The pipeline is a set of **boundary providers** wrapping firmware/vendor code; the audited edge is explicit.
- Decode/render sessions are **machines with states** ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md)): configured → running → flushing → drained.
- Frame/audio buffer descriptors crossing a boundary use ordinary numbered schemas and selected layout/codec policies ([wire protocols](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)).
- Omega gives no GPU memory-isolation model for shared devices; that isolation guarantee is a Cathedral + hardware obligation.

## Open Questions

- Is a clean capability model achievable over today's monolithic vendor GPU stacks, or does the first target need a simpler/software path until drivers cooperate?
- Does Cathedral support hardware DRM at all in v1, given the trust it demands?
- Can the display controller be driven cleanly enough that the compositor and its trusted path never depend on the heavy GPU stack, even when no accelerated GPU driver is present?

## Related
- [[windowing_and_compositor]] — window management above this pipeline.
- [[driver_model]] — isolating the GPU/decode vendor driver.
- [[scheduler_and_resources]] — frame scheduling and GPU budget.
- [[power_management]] — power-aware rendering.
- [[capability_model]] — screen capture and GPU access as held capabilities.
- [[kernel_architecture]] — the hardware-isolation wall the GPU blob sits behind.
- [[audio]] — the audio pipeline, now its own chapter.
- [[filesystem_as_database]] — content-addressed fonts and locale data, and the no-special-folders model.
