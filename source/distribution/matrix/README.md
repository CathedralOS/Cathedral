# Stock matrix behavior in Omega

Start at [session/main.omg](session/main.omg) for the stock service-turn sequence,
including progress, work priority, waiting and event dispatch. The
[session boundary](session/README.md) leads into its Rust mechanisms.
[requests/main.omg](requests/main.omg) selects request operations from authenticated
caller facts; [the request boundary](requests/README.md) explains reply custody.
[realm/main.omg](realm/main.omg) sequences realm discovery and workspace,
picker and background setup; [the realm boundary](realm/README.md) leads to host resources.
[resources/main.omg](resources/main.omg) chooses stock role offers, profile
ceilings and surface preferences; [the resource boundary](resources/README.md)
leads to the host's actual grants and allocations.
[policy/main.omg](policy/main.omg) dispatches the temporary native reference
bridge to [configuration](configuration/README.md), [lifecycle](lifecycle/README.md),
[launcher recovery](recovery/README.md), [local startup recovery](startup/README.md)
or [ordinary stored-program launch](launch/README.md) and
[existing-app workspace selection](workspace/README.md).
The chooser imports the configuration codec directly. The stock F5 matrix runs
these policies through [native/main.omg](native/main.omg), a single compiled export.

Configuration owns saved desktop settings and placement calculations. Lifecycle
owns loss aggregation and the wait/refuse/stop decisions for ordinary close.
Recovery owns bounded retries and the asynchronous read/wait/admit workflow,
with ticketed completions and fallback decisions.
Startup owns local picker/background retry budgets and selection-failure recovery.
Launch owns target admission, role classification, read/admit/bind/place/publish
sequencing and partial-failure rollback.
Workspace owns switch/wait/activate sequencing and restoration after a late
accepted commit. Cancellation drains the scene before finishing or restoring.
It also owns the app catalogue, instance-ticket allocation, current selection and
fallback choice after retirement; Rust retains opaque state and exact task custody.
Ordinary reads and storage binding yield; admission/publication still use
synchronous host mechanisms.
Rust still authenticates callers, serializes IPC, retains exact task/endpoint
identity, drains late replies, progresses asynchronous parent work and reclaims
tasks/resources. Service-turn orchestration, request-family routing and owner
success consequences now execute in Omega, as do owner child-control dispatch and
launch target/role choices. Realm discovery and configuration/picker/background
setup sequencing, stock resource offers and profile ceilings also execute in Omega.
Stock service offers and live-layout plans also execute in Omega; common interface
matching lives in platform contracts. Image declaration decoding and local
world/resource operations retain Rust behavior. These are bounded migrations
of distribution behavior, not a new platform API or the completed Omega OS.
