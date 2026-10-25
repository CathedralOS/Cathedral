# Stock matrix behavior in Omega

Start at [policy/main.omg](policy/main.omg), which dispatches the Rust reference
bridge to [configuration](configuration/README.md) or [lifecycle](lifecycle/README.md).
The chooser imports the configuration codec directly. The stock F5 matrix runs
both policies through [native/main.omg](native/main.omg), a single compiled export.

Configuration owns saved desktop settings and placement calculations. Lifecycle
owns loss aggregation and the wait/refuse/stop decisions for ordinary close.
Rust still authenticates callers, serializes IPC, retains exact task/endpoint
identity, drains late replies, progresses asynchronous parent work and reclaims
tasks/resources. The matrix service remains Rust; these are bounded migrations
of distribution behavior, not a new platform API or the completed Omega OS.
