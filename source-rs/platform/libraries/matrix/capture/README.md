# Bounded capture mechanisms

Start at [capture.rs](capture.rs): an owner grants one local source binding to one
recipient endpoint, begins a bounded request, commits a completed masked result,
and retains that result until transfer succeeds or the lease is retired.
[Geometry](geometry.rs) validates an immutable crop/mask plan.
[Rasterization](raster.rs) produces packed RGBX output from stable source layers.

These provider mechanisms are exercised by the installed
[Split provider](../../../../distribution/applications/split/session/capture/capture.rs)
and a separate [capture viewer](../../../../distribution/applications/capture-demo/main.rs).
The [capture protocol](../../../contracts/capture.rs) and
[recipient client](../../user-runtime/capture.rs) define their IPC boundary.
Only Split explicitly offers this service; ordinary app offers remain unchanged.
The demo uses a fixed owner-selected crop/mask over Notes, not general app-published
privacy metadata or recursive source readiness.

## Provider obligations and sequence

1. Derive source/recipient identities from accepted endpoint incarnations and
   the source's current local view epoch. Authorize the crop and masks under
   owner policy; `Plan` validates geometry, not permission. Do not accept a PID
   or source slot from an untrusted packet as authority.
2. Fund destination pages, source reader retention and work before `Lease::begin`.
   The lease allows one pending request or retained result, not an unbounded
   queue. The caller owns actual allocation/accounting. Time and request lifetime
   come from the provider's clock/policy.
3. Use [World::request_frame](../world/demand.rs) to add independent production
   demand and wake the source. The source sees `FRAME_REQUESTED` in its own POLL,
   queries FRAME_DEMAND, publishes its current scene, and sends FRAME_READY with
   that ticket. Lease request IDs and source frame tickets are separate; the
   provider associates them and may later coalesce several consumers.
4. Authenticate the acknowledgment and pin a coherent accepted scene before
   dispatching another publication. The handshake is cooperative freshness, not
   a display fence or proof that a hostile source did new work. A timeout alone
   does not prove a fresh image exists. Matrices need descendant coordination
   before acknowledging a recursive frame; that adapter is not implemented.
5. Read only sealed/pinned source backing. `Plan::render` validates the complete
   input before writing, crops in source-local coordinates, and applies fixed
   black masks before sampling pixels. Masking cannot be overwritten by a later
   layer or cursor. RGBX padding and destination page slack are zeroed. Never
   share an unmasked source attachment with the recipient. Freeze the successful
   private output before calling `Lease::publish`.
6. Release frame demand once source/result custody is established, or on timeout,
   refusal or recipient loss. Rendering demand neither reveals the source nor
   changes input selection. The source's display demand may remain absent.
7. Borrow `Lease::ready` for attempted delivery. A full peer queue retains the
   same funded result; it must not trigger another render/allocation. Release
   after successful transfer or cancellation. Reconcile participant/view changes
   before dispatch: source or recipient death/replacement retires the grant and
   retained result. Already transferred read rights follow kernel custody, and
   are not revoked by deleting this provider's reference.

Provider reconciliation is required; the pure library does not inspect processes
or clocks itself. Pending-request expiry removes demand but is not process death.
A ready result remains charged until delivery/cancellation/death; its reader is
not pulled out from under a live recipient. Tickets do not restart on regrant;
exhaustion refuses new requests. Mask changes require grant replacement, retiring
old pending work rather than pairing an old scene with new metadata.

## Current bounds and evidence

One plan has at most eight masks and a 32,768-pixel crop (128 KiB RGBX); source
geometry is bounded to 4096 per dimension. Rasterization accepts at most 64
ordered solid/image/cursor layers. Empty space is black. Inputs must be coherent
and authorized by the caller; visibility is deliberately not a raster input, so
an authorized hidden presentation can be captured. This scalar software path
makes no latency or GPU claim.

Host tests exercise sibling/endpoint refusal, view replacement, stale ticket
cleanup, clock wrap/expiry, counter exhaustion, destination bounds, stride and
crop handling, mask overlap, cursor transparency, zeroed padding, retained-result
lifetime, and the composed hidden-frame/result/release sequence. Installed IPC
checks live in `tools/boot-harness-rs/desktop_test.py --capture`: a separate
recipient inspects actual redaction/padding bytes and publishes only the masked
image while the source remains hidden. Lifecycle variants use `--capture-case`;
see [the current handoff](../../../../../TASKS_RUST.md) for verified runs and limits.
