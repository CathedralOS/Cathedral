# Rendering evidence, conflicts and acceptance

The Rust lab measures narrow mechanisms. It does not implement the proposed
retained scene, resource service or GPU presentation architecture.

Status: supporting evidence for [rendering and composition](../0000_rendering_and_composition.md).
This page owns the evidence boundary and acceptance criteria. Owner-level choices
remain in the linked decision queue rather than being duplicated here.

## Existing evidence

| Mechanism | Evidence | What it does not establish |
| --- | --- | --- |
| Copied drawing requests and shared pixel input | [Rust display contract](../../../source-rs/contracts/display.rs), [provider](../../../source-rs/platform/services/display/surface.rs) | Production drawing vocabulary, retained resources or frame atomicity |
| Private pages, sealing and accepted readers | [Page contract](../../../source-rs/contracts/memory.rs), [runtime wrappers](../../../source-rs/platform/libraries/user-runtime/memory.rs) | General multi-reader delegation, revocable live borrows or GPU fences |
| Bounded retained scenes and connection-scoped drawing | [Two-client compositor experiment](../../../source-rs/distribution/lab/compositor/README.md) | Cross-process Matrix delegation, protected output, general resource custody or tear-free presentation |
| Flattened leaves versus ancestor copies | [Rendering investigation](../../../source-rs/distribution/lab/rendering/README.md) | Real nested Matrix IPC, arbitrary effects, alpha or GPU performance |
| Checked row spans and native stores | Same investigation's host tests and QEMU bounds probe | Native sub-page isolation or proof-carrying admission |
| Handle and peer lifetime failures | [Memory fixture](../../../source-rs/distribution/lab/memory/main.rs) | Persistent principal accounting, retained cache eviction or asynchronous resource loading |

The rendering investigation records host byte counts, timing scope and guest
mapping/IPC counters. Its repeated opaque quadrant workload cannot support a
general desktop performance claim. Measurements and reproduction commands stay
with that fixture so this proposal does not become a second benchmark ledger.

## Design conflicts

| Existing position | Conflict or unresolved seam | Decision owner |
| --- | --- | --- |
| Media chapter puts all glyph rasterization in app libraries and says the compositor never sees glyphs | Optional platform glyph/path execution moves part of rendering across that boundary | [Platform drawing and text placement](../../../OWNER_QUESTIONS.md#platform-drawing-and-text-placement) |
| Recursive compositor receives frames and presents a result upward | Forwarding scenes/resources needs delegated authority and host interception without mandatory intermediate images | [Scene submission](scene_submission.md#authority-through-the-host-chain); protocol acceptance evidence below |
| OS components may share a proved domain or use default MMU isolation | Direct target access changes fault and confidentiality boundaries | [Default isolation](../../../OWNER_QUESTIONS.md#default-isolation-boundary-for-proved-os-components), [writable spans](../../../OWNER_QUESTIONS.md#writable-row-spans-and-component-admission) |
| Revocation wording allows redeemed operations to finish | Accepted mappings and queued GPU work can outlive a producer or a request to stop | [Endpoint and accepted-lease lifetime](../../../OWNER_QUESTIONS.md#endpoint-transport-and-revocation-of-pending-ipc) |
| Resource costs are bounded but image/cache sponsorship is unspecified | App-funded storage can remain pinned by other principals after producer failure | [Residency and sponsor liability](../../../OWNER_QUESTIONS.md#render-resource-residency-and-sponsor-liability) |
| Heavy GPU service is optional and contained; trusted output remains available | Accelerating the common renderer changes the output trust/recovery dependencies | [Execution and presentation](execution_and_presentation.md#presentation-and-protected-output); device-profile evidence below |

These links identify questions, not exemptions from existing authority rules.
The proposal does not settle the separate [replaceable chrome question](../../../OWNER_QUESTIONS.md#replaceable-distribution-versus-permanent-os-chrome).

## Acceptance evidence

Each accepted slice needs a bounded contract and evidence for its own guarantees.
A software scene protocol can be evaluated before a GPU backend; that acceptance
would not establish GPU import, scanout or device-failure claims.

| Slice | Required evidence before its guarantees are claimed |
| --- | --- |
| Scene publication | Atomic invalid-batch rejection, stale revisions, bounded graph validation and resource generations |
| Nested authority | Parent clip/transform composition, forbidden sibling references, host interception and protected-layer denial across real IPC |
| Resource custody | Publication immutability, producer/reader death, delayed completion, stale providers and release only after the final actual user |
| Demand loading | Slow, failing and malicious providers cannot block unrelated output; missing-resource policy is observable and tested |
| Accounting | Large granted workloads succeed; metadata floods, pinned-byte retention, cache growth and parent-budget evasion remain bounded |
| Text/images | Specified shaping/raster responsibilities, representative scripts and sizes, malformed assets and decoded-size limits |
| Software rendering | Equivalent pixels across nesting, retained updates, clipping, transparency and defined intermediate effects |
| GPU/presentation | Supported import formats/devices, completed-image reuse, actual copy/upload counts, scanout eligibility, protected output and device-failure custody |

Budgets and fallback semantics need owner acceptance before conformance is
meaningful. Candidate example limits are not universal product ceilings.

## Experiments that discriminate between choices

Useful comparisons include a mostly static editor, a scrolling document, an
animated image view, and a custom-rendered full-frame producer. Vary Matrix depth
without changing visible content. Add a subtree opacity/effect case to expose
when an intermediate image is actually necessary.

Record metadata bytes, pixel copies, uploads, allocated and pinned bytes, cache
hits, derived storage, IPC deliveries, CPU/GPU work and presentation latency.
Include idle cost and missed deadlines. Compare cold assets with warm caches and
large grants with backpressure. Host CPU timing is not a GPU or scanout benchmark.

These are evaluation criteria, not a second execution board or a commitment to
implement every experiment before choosing an initial bounded slice.

## Documentation changes on acceptance

Accepted rendering behavior belongs in `spec/human_surface/rendering.md`, with
authority, IPC and compositor/seat rules reconciled in their respective owners.
Machine-readable contracts own the selected encodings. Update the graphics and
windowing rationale in the same change, particularly the text-placement conflict.
Implementation coverage remains independent of specification coverage.

Remove resolved owner questions after their answers have a contract owner.
Remove incorporated proposal sections or the full proposal when superseded.
Keep benchmark reports and code-level limitations beside their implementations.
