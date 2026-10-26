# Experimental display provider

Owns bounded software drawing into one boot-granted linear GOP framebuffer.
In direct mode, the separately compiled userspace service exposes dimensions,
clear, rectangle and printable ASCII text requests over shared-memory IPC. Text uses
the platform's original bitmap font; every packet and its entire geometry are checked before
any device write. It accepts no physical addresses.

The data channel also accepts `BLIT`: a sealed region token plus destination
rectangle. The provider accepts a bounded read-only lease, checks the complete
pixel payload and geometry, copies little-endian `0x00RRGGBB` pixels into scanout,
and drops the lease before success or error completion. The kernel binds the
token to this peer; a token alone does not grant another task access. Control
messages cannot request BLIT. This is a CPU blit into GOP, not GPU rendering or
zero-copy scanout. Producers choose the content and retain their owner reference
until completion. Lost replies require caller recovery, not implicit reuse of
an outstanding region.

A private control connection serves the designated supervisor; a boot-approved
client link serves the application. The provider multiplexes both with bounded
readiness waits and accepts a fresh data connection after app replacement.
Client loss leaves the provider and its control connection alive. Linked service
composition and the single-peer smoke fixture require explicit clock grants and
funded mailbox writer/reader pages plus notifications. The fixture's supervisor,
drawing client and independent observer also retain shared coordination streams;
see its [proof guide](../../../distribution/lab/display/README.md).

Distribution clients choose colors, layout and restart policy. Whole-screen
drawing in direct mode remains a lab capability, without a trusted prompt path,
capture API or stable surface contract. `lab` adds boot-controlled smoke faults
and mapping probes; `recovery-lab` adds boot-selected startup failures and private
control-channel failure injection. Neither is included in ordinary builds, and
non-control client messages cannot trigger those handlers.

## Entrances and ownership

[main.rs](main.rs) selects the protocol: [direct entrance](direct/direct.rs) serves immediate
[drawing requests](../../contracts/display/display.rs), while
[compositor entrance](compositor/compositor.rs) serves the retained
[composition protocol](../../contracts/display/composition.rs). Both use
[surface entrance](surface/surface.rs) for the boot-granted framebuffer. Direct-mode smoke
probes and recovery hooks live under `direct/`.

The compositor loop dispatches requests, connection loss and deferred delivery.
Its [composition state](composition/composition.rs) shows row order: background, labels,
application scenes, then cursors. Descend into [scene entrance](composition/scene/scene.rs)
for snapshots, hosting/clips and rasterization, or [input entrance](composition/input/input.rs)
for pointer lifetime, routing, cursor damage and keyboard focus. These policies
remain host-testable without device access or IPC. The runtime handlers under
`compositor/` bind that state to authenticated connections.

Distribution clients choose content and request presentation; the provider
validates authority, placement and clipping. This retained lab mode is still an
experimental protocol, not the complete compositor design.

## Root matrix connection

Normal boot supplies separate presentation-owner and bootstrap connections.
The bootstrap stream configures dimensions/labels and the restart screen without
scene publication or input-injection authority. The
[root adapter](../../../distribution/matrix/service/parent/display.rs) establishes
its own [shared service connection](../../libraries/user-runtime/connection/README.md)
once; steady-state commands use shared frames and notifications, and
images use correlated typed immutable grants. The control bit comes from the
installed connection, never a caller-supplied frame field. Pixel validation,
clipping, retained-image lifetime and presentation commit remain with the display
provider. Losing the owner retires this connection and ends the provider normally.

Presentation and selection scene updates yield through the matrix's event loop.
Each shared stream consumes an explicit private page, reader page and notification
allowance; root profiles reserve matching client capacity. Direct-mode smoke and
ordinary nested matrix hops use shared transport too. Older positional compositor
fixtures still need migration. This does not imply faster software rasterization
or GPU support.
