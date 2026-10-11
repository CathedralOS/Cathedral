# Rust implementation map

This map connects implemented mechanisms to their integrated consumers, proofs
and remaining restrictions. The [lab guide](README.md) owns detailed ABI/build
instructions. [Owner questions](../OWNER_QUESTIONS.md) own unresolved semantics.

## Current integrated path

Start at [distribution desktop policy](distribution/lab/matrix/desktop.rs).
It admits display/input adapters and lends child launch grants to a root matrix.
The root matrix admits an inner matrix and an outside app; the inner admits its
own app. Both run the same [matrix service](platform/services/matrix/service.rs).
Only the root adapts physical input and compositor output.

```powershell
python tools/boot-harness-rs/run.py --matrix-desktop-test --release --timeout 120
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/matrix-desktop.json --release --window
```

Click either colored app, then Enter to change its color. The left app is nested;
the right app is its outer sibling. Startup exercises inner-subtree replacement
and checks that old pixels disappear while the sibling survives. The automated
host test adds real mouse/keyboard injection and framebuffer checks.

The earlier `session.json` desktop remains a richer compositor/input regression
fixture. The default distribution profile still starts the storage/status demo.
Neither is presented as using the new matrix-owned lifecycle. These fixtures
remain until their extra behavior has moved into the integrated path with tests.

## Mechanisms, consumers and evidence

| Mechanism / owner | Integrated consumer | Proof | Remaining restriction |
| --- | --- | --- | --- |
| [Address spaces, admission, faults](kernel/core/user_tasks/session.rs) | All separately compiled services/apps | `--smoke`: rollback, faults, stale tasks, frame/heap reclamation | One x86-64 CPU; bounded boot-selected ELF/task arena |
| [Dynamic IPC and startup bundles](kernel/core/ipc.rs) | Matrix-child, root-display and physical-source connections | `--ipc-test`: moves, rejected bundles, death, quotas | Small copied messages; accepted moves can outlive their creator |
| [Backed IPC reservations](kernel/core/ipc/budget.rs) | Every initial/launch account, including matrix children | Host tests fill all 32 slots; `--ipc-test` exhausts restarted creators while a sibling fills its reservation | Static launch accounts; no hierarchical subdivision or sponsorship transfer |
| [Recursive launch loans](kernel/core/supervision/delegation.rs) | Matrices own direct children, lend budgets further, reclaim subtrees | `--ipc-test` plus matrix replacement tests | Approved executables/slots; not a general CPU/RAM sponsorship contract |
| [Matrix lifecycle](platform/services/matrix/service/children.rs) | Same provider above apps and another matrix | `--matrix-test`: replacement preserves an outside sibling | Two direct children; distribution supplies policy through owner-only control |
| [Local viewport, focus and capture](platform/services/matrix/world.rs) | Nested and sibling visible apps | Host policy tests; `--matrix-test`; `--matrix-desktop-test` physical focus | Five keys; bounded queues; app polling and synchronous upward refresh |
| [Clipped surface commands](platform/services/matrix/world/surface.rs) | App rectangles cross actual matrix boundaries to the existing compositor | Host clipping/epoch/z-order tests; scanout and teardown checks | One primitive per path; current renderer supports two matrix levels; no images/text surfaces yet |
| [Retained compositor and hardware input](platform/services/display/compositor.rs) | Root adapter uses existing scanout/cursor and shared input decoders | `--matrix-desktop-test`; `--session-test --pressure` | Existing richer scenes/hosting remain in the earlier session fixture |
| [Private/shared page lifetime](kernel/core/regions.rs) | Existing rendering/storage labs | Host policy tests and `--smoke` | Per-slot page limits and boot-approved sharing; no general shared-object IPC transfer |
| [Object storage and recovery](platform/services/storage/service.rs) | Default status/counter distribution | Storage host tests; `--storage-test` | Not yet an attenuated matrix namespace; not the custom database filesystem |

## Kernel boundary audit and next steps

The matrix protocol exposes local state only. That is narrower than a claim of
whole-system noninterference. The handle-locality pass now covers task, clock, link and page rights as well as
IPC. Host tests compare names and mapping addresses under unrelated graph/arena
activity. The remaining boundary work is recorded below:

| Surface | Current finding | Next implementation boundary |
| --- | --- | --- |
| IPC handles | [Grant table](kernel/core/ipc/grants.rs) supplies local, non-reused names | Retain this model when generalizing other capabilities |
| Task/launch/peer handles | [Caller-local names](kernel/core/names.rs); [admission reservations](kernel/core/user_tasks/session/lifecycle/grants.rs) and reserved fresh loan-return names | Keep this boundary when replacing fixed executable/slot admission |
| Clock and static links | [Clock](kernel/core/deadline.rs) and [link](kernel/core/link.rs) names refresh on recipient admission; unchanged peers retain their names | Separate timing authority from waiting/deadline mechanics; hardware timing remains visible |
| Page handles | [Region grants and virtual slots](kernel/core/regions.rs) are caller-local; producer-local offers require the receiving link and install a separate reader name | Generalize beyond boot-approved sharing; preserve accepted-reader lifetime |
| IPC capacity | Backed launch-account reservations; surviving channels stay charged across creator replacement; caller-local budget query | Generalize account subdivision and explicit funding transfer; retained endpoints can pin their original account |
| CPU / RAM | Round-robin processes and fixed page grants; admitting more children adds runnable entities | Parent-budget subdivision, separate accounting from reservation, test sibling progress under exhaustion |
| Admission | Executable and resource ceilings come from reserved boot slots | Admit an explicitly granted executable within a budget, with complete rollback and no ambient image lookup |
| Devices | Concrete framebuffer/PS2/ATA grants, no arbitrary device discovery | One reusable userspace interrupt/MMIO boundary, proved with another driver; DMA needs a separate ownership/isolation design |

Immediate integration follow-ups are event-driven matrix input, retained local
scene replay after placement changes, and replacing fixed surface keys with
budgeted surface objects. Current apps repaint their small primitive periodically;
the compositor suppresses identical updates, but this is not the final idle or
latency design. Root-adapter recovery/trusted-path behavior remains an owner-level
policy question. SMP, GPU support and the database filesystem follow these
boundaries rather than becoming prerequisites for exercising them.

Local names are scoped by caller and API namespace, not globally unique bearer
secrets. The same number can designate unrelated rights in two processes. Passing
it as message data installs no authority. Counters survive task-slot reuse so old
names cannot resurrect a replacement's grants; unsuccessful admission may burn
reserved names. This does not conceal timing, resource contention or all local
admission history. No claim of whole-system noninterference follows from it.
