# Owner child-control policy

Start at [main.omg](main.omg): require an authenticated owner and bounded child
count, recognize loss/unhandled work before loss exclusion, then choose routed
or local work. [operation.omg](operation.omg) names host facts, decoded requests,
local commands and decisions. A selected slot is a local position, not authority.

[wire.omg](wire.omg) decodes the actual Matrix packet: canonical loss takes
precedence over the shared imports opcode; routed requests retain their first
hop and configuration classification. [local.omg](local.omg) checks per-operation
limits, canonical syntax and occupied/standing state. Invalid requests never gain
authority by being decoded. Pending requests block forwarding only on their own
hop; unrelated owner work can still reach its fallback during a loss survey.

The [native adapter](../../../bridge/policy/requests/control.omg) encodes host
operation tags and decodes the host envelope. The
[Rust owner handler](../../../../../../source-rs/distribution/matrix/service/children/control/owner.rs)
retains exact tasks, grants, authenticated connections and asynchronous replies.
The existing [compiled tests](../../../../../../source-rs/distribution/matrix/tests/control.rs)
check these decisions against the Rust protocol and resource limits.
