# Retained compositor experiment

Two isolated clients submit bounded scenes to a userspace compositor. Their
connections own content; the distribution supervisor owns placement and recovery.
This is an experimental protocol, not an accepted Cathedral surface contract.

From the repository root:

```text
python tools/boot-harness-rs/run.py --compositor-test
python tools/boot-harness-rs/run.py --compositor-test --release
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/compositor.json --window
```

The custom profile leaves the ordinary status distribution available. Its lab
supervisor runs drawing and failure probes, then leaves two labeled panels on
screen. It has no mouse or interactive window controls. The automated test saves
`build/boot-harness-rs/compositor-test/display.png` and checks every scanout pixel.

## Follow execution

Start at [main.rs](main.rs), then [supervisor.rs](supervisor.rs). Startup admits a
framebuffer-owning display provider and two copies of the client executable.
[Drawing checks](supervisor/checks.rs) exercise scopes and snapshots;
[recovery](supervisor/recovery.rs) fails participants while preserving peers.
[Client](client.rs) owns submission and hostile requests; [scene](scene.rs) owns
appearance. The [profile](../../../../tools/boot-harness-rs/profiles/compositor.json)
binds devices and connections.

The receiving [display entrance](../../../platform/services/display/main.rs)
selects [compositor execution](../../../platform/services/display/compositor.rs)
from its boot-supplied mode. Follow [request dispatch](../../../platform/services/display/compositor/requests.rs)
for authority, [scene policy](../../../platform/services/display/scene.rs) for
validation and drawing, and [surface](../../../platform/services/display/surface.rs)
for bounded device writes. The [wire contract](../../../contracts/composition.rs)
imports neither implementation.

## Bounds and lifetime

Each of two clients has at most eight nodes, one 16x16 RGB image and a scope no
larger than 256x256. Nodes are rectangles, existing ASCII text, images or groups.
Earlier-group parent references prevent cycles. Checked local translations and
intersected ancestor clips resolve directly into drawing; groups have no pixel
buffers. Later nodes and the second client draw above earlier ones. The control
connection alone sets client placements, background and lab readback queries.

One sealed page carries a full snapshot. The kernel's optional
`MEMORY_MAP(handle, service_link)` verifies its producer against the actual
client link before acceptance. A client cannot induce the service to consume a
neighbor's offered page by guessing its handle. Existing zero-link acceptance
remains available for older one-client lab protocols.

Validation uses fixed scratch storage before swapping the whole scene into the
client's retained state. Invalid geometry or stale revisions preserve committed
content. The small image is copied into bounded service storage during acceptance;
the read lease is released before replying. This is a bounded-copy experiment,
not demand-loaded app-funded resource custody or zero-copy presentation.

Revisions belong to connection incarnations. The service observes disconnects,
removes that instance's retained content, and redraws exposed peers. A replacement
starts at revision one with a new connection. A queued request can finish before
disconnect is observed; this does not define production revocation timing.
The generic server rotates clients and control, and full reply queues do not
block the other client. Every accepted snapshot has bounded decoding and drawing
work, though no real-time or fair CPU-share guarantee is established.

All drawing writes GOP directly. Snapshot acceptance is atomic as scene state;
scanout writes are not a tear-free frame commit. There is no vblank scheduling,
GPU, alpha, protected prompt, cursor, input focus, delegated cross-process Matrix
tree or stable surface ABI. Local groups exercise nesting without claiming those
larger authority contracts.

## Evidence

Host tests compare every pixel around nested clips and scope edges, reject
malformed scenes, and check stale revisions and disconnect incarnation handling.
Kernel policy tests check producer authentication without consuming foreign offers.

QEMU checks actual device pixels after scope clipping, movement, overlap and
exposure; rejects stale and foreign handles; and repeatedly releases/reallocates
a client's single-page budget. It crashes, spins and floods one client, replaces
it, then replaces the compositor while both clients survive and reconnect.
The host independently reconstructs the final image and compares every pixel.
This bounds the tested resource reuse; the profile remains live for inspection
and does not claim a new whole-session frame/heap baseline measurement.

The [rendering proposal](../../../../wiki/proposals/0000_rendering_and_composition.md)
continues to own candidate production behavior. The lab supplies narrow evidence
without resolving its text, admission, resource residency or trusted-chrome choices.
