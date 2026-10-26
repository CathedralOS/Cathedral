# Stock matrix behavior in Omega

Start at [policy/main.omg](policy/main.omg), which dispatches the Rust reference
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
Launch owns read/admit/bind/place/publish sequencing and partial-failure rollback.
Workspace owns switch/wait/activate sequencing and restoration after a late
accepted commit. Cancellation drains the scene before finishing or restoring.
It also owns the app catalogue, instance-ticket allocation, current selection and
fallback choice after retirement; Rust retains opaque state and exact task custody.
Ordinary reads and storage binding yield; admission/publication still use
synchronous host mechanisms.
Rust still authenticates callers, serializes IPC, retains exact task/endpoint
identity, drains late replies, progresses asynchronous parent work and reclaims
tasks/resources. The matrix service remains Rust; these are bounded migrations
of distribution behavior, not a new platform API or the completed Omega OS.
