# Rendering execution and presentation

Rendering computes pixels. Presentation selects which completed images reach an
output. The scene contract should preserve that separation across software and
GPU backends.

The accepted [presentation contract](../../spec/human_surface/rendering.md) now
owns backend-independent forwarding, completion and fullscreen boundaries.
The concrete mechanisms below remain alternatives under
[rendering and composition](../0000_rendering_and_composition.md).
This page owns backend and presentation choices. Resource accounting and release
are owned by [resource custody](resource_custody.md).

## Cost by path

| Path | Pixel work and authority |
| --- | --- |
| Bounded drawing descriptions | Trusted renderer writes its target under validated clips; no app raster buffer is required |
| Finished image in compatible shared backing | Import avoids transport copying; composition still samples or copies into a target when needed |
| Pixel bytes in copied IPC | Moves bulk pixels through the transport before target writes |
| Image at every ancestor | Repeats composition for each materialized subtree |
| Eligible direct scanout | Display controller reads the completed image; no intermediate composition image required |
| Hardware planes | Device combines eligible images within its placement, format and plane limits |

Zero transport copies, zero intermediate images and zero composition work are
different claims. A copy does not rerun a game's geometry and lighting, but still
costs bandwidth and time. GPU composition also does work even when it uses the
producer's existing image without a CPU readback.

## Software execution

The accepted [final-target writer boundary](../../spec/human_surface/rendering.md#final-target-writers)
keeps independent producers on commands or their own images. Row partitioning
below is internal renderer work, not a pending ordinary-app admission choice.

The trusted software renderer can execute resolved commands into a common back
buffer. It can delegate checked row borrows internally while retaining control
of target custody. A failed frame can be withheld without modifying the visible
frame when the presentation path provides a separate back buffer.

Native code with a writable page can address bytes outside an advertised
sub-page row span. Drawing order cannot fix ownership: the last hostile writer
can corrupt earlier output, and later drawing cannot undo a prior unauthorized
read. A trusted correction pass needs authoritative source content and adds work.

The current Rust framebuffer path copies into the firmware-provided GOP target.
It has no production back-buffer swap or direct-scanout protocol. Its row lab is
an offscreen computation test, not a safe lease of visible scanout.

### Whole-page leases versus pixel rectangles

The accepted [IPC mapping boundary](../../spec/communication/ipc.md#shared-regions-and-capacity)
permits disjoint whole-page mappings within a larger allocation. The compositor
could see the full allocation while each writer sees only its authorized pages.
This remains an optional research mechanism, not an accepted surface ABI or a
blocker for the ordinary command/image route. It cannot justify writable sub-page
leases to independent apps.

For example, with 4 KiB pages, a page-aligned linear image of 1,024 four-byte
pixels per row and no row padding has one page per row. Exclusive whole-row
leases fit that layout; independent left/right half-row leases do not. Ordinary
rectangles can share pages with unrelated pixels. Guard pages do not solve that
sub-page boundary. Padding or page-sized tiles may help ownership, but their
layout must also suit the renderer and display provider or require conversion.

Separate page-isolated surfaces can instead be forwarded as authorized references
with placement and clipping, without materializing an image at every ancestor.
This forwarding route is accepted: some effects require intermediate targets, and sharing
backing avoids transport copies without eliminating final composition work.

Spatial ownership and temporal custody are independent. Double buffering alone
does not stop a hostile producer modifying a submitted image. Stable presentation
requires enforceable publication and reuse rules covering all writable aliases
and device access. The page-lease route is not implemented by the current row lab.

## GPU execution

Drawing descriptions can feed a GPU backend without exposing GPU command buffers
to ordinary UI clients. The backend chooses tessellation or compute work, glyph
representation, image sampling and clipping. The public rights remain about
resources and drawing scope rather than physical row addresses.

GPU images may use implementation-specific tiled storage rather than linear
rows. [Vulkan's tiling definitions](https://docs.vulkan.org/refpages/latest/refpages/source/VkImageTiling.html)
make this distinction explicit. A backend may require conversion or staging when
an imported image's format, layout or device is incompatible. A portable protocol
should report supported routes instead of promising zero-copy import everywhere.

A trusted backend can use scissors, stencil or shader logic to enforce its
drawing clips. A [scissor command](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetScissor.html)
is not by itself a sandbox for arbitrary GPU programs. A caller controlling
unrestricted commands can change that state; other resource writes also need
confinement. Validated command/resource access and device memory isolation are
separate obligations from clipping ordinary drawing.

Custom engines retain a completed-image path. They render through an admitted
GPU provider into their own authorized resources, then present a version under
the same lifetime model. No child receives the root target merely because it
uses the GPU. Neither CPU proof checking nor a process label implies GPU safety.

## Scheduling and completion

Ready resources become eligible for rendering. Render completion makes their
output eligible for presentation. Presentation completion and the end of all
other readers determine when storage can be reused. A queued command submission
is not a completion signal.

Hardware work needs an enforceable custody interval plus synchronization.
DMA-BUF's [buffer/fence separation](https://docs.kernel.org/driver-api/dma-buf.html)
is a reference model, not a selected Cathedral ABI. Merely promising to wait on
a fence does not contain a malicious producer that can still overwrite backing.

The platform admits bounded outstanding work and preserves progress for other
clients. Hard GPU deadlines depend on the chosen device's preemption, fault
isolation and reset behavior. The proposal does not infer those guarantees from
a software work estimate or claim a GPU implementation exists.

Resource waits happen before final drawing. Late frames may be skipped under a
defined presentation policy without dropping dependent scene updates or releasing
resources still in use. Secure hide/lock and cancellation outcomes need their
own semantics, identified in [evidence and acceptance](evidence_and_acceptance.md).

## Presentation and protected output

The display provider can select direct scanout, hardware planes or a composed
target according to hardware support and current scene requirements.
[DRM/KMS](https://docs.kernel.org/gpu/drm-kms.html) describes framebuffer/plane
composition and validation of modes, formats, placement and shared resources.
Fullscreen alone does not establish eligibility.

Protected prompts, capture policy, multiple outputs, effects or color conversion
can require another route. A low-latency grant cannot remove the platform's
protected interaction boundary. Direct scanout hands an eligible image to the
display provider; it need not hand unrestricted display-device authority to the app.

Software fallback is an architectural goal, not evidence that CPU and GPU targets
are interchangeable. Display-controller recovery, GPU loss and trusted prompt
availability remain questions for the device/trust profile in [[media_and_graphics]].
