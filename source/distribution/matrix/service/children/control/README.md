# Close and loss policy

[main.omg](main.omg) owns the stock matrix's conservative loss decisions. A survey
starts only when no previous survey, control request, storage/admission operation
or recovery is pending. Every successfully queried child must answer. Failed sends,
errors, timeouts, dirty answers and noncanonical replies make the aggregate uncertain.
A clean sibling cannot erase uncertainty; recovery appearing before completion also
prevents a clean result. Only the boolean aggregate leaves the matrix.

An ordinary close waits while the matrix's own upstream/storage work prevents
teardown. Once the host has a current answer, Omega chooses refuse, stop, or already
stopped. A live child is stopped only after an exactly canonical clean reply:
`[0, 0, 0, 0, 0, 0, 0, 0]`. A broken connection alone is never proof of death.
This is an advisory observation, not a transaction freezing subsequent app work.
Explicit authorized force-stop remains a separate operation.

The [Rust adapter](../../../../../../source-rs/distribution/matrix/policy/lifecycle.rs)
passes only values. Its survey stores two policy words; they hold pending-child
bits and accumulated uncertainty, never a PID or capability. The
[loss orchestrator](../../../../../../source-rs/distribution/matrix/service/children/loss.rs)
retains the authenticated requesting endpoint and sends queries. The
[control loop](../../../../../../source-rs/distribution/matrix/service/children/control/control.rs)
retains FIFO custody, drains stale replies before accepting a new answer, and
observes the exact kernel task outcome. Omega never selects a child from untrusted
message metadata, reads an endpoint or performs teardown.

The [Omega adapter](../../../bridge/policy/lifecycle.omg) translates between
`LossSurvey`, `LossAnswer`, `SurveyConclusion`, `CloseDecision` and the Rust host.
The private 128-word native packet uses operation in word 0, facts in 1-2,
survey state in 16-17 and a complete child answer in 24-31. Operations 3-6 begin,
record sends, consume answers and finish a survey; 7 decides close completion;
8 gates close while teardown is busy. This bridge is not an IPC wire contract.
The five child slots are the current stock distribution's composition limit.

Compiled host tests cover every child subset, completion order, admission/recovery
conflicts, failed sends, transport errors and malformed reply words. The ordinary
F5 desktop test exercises real clean/dirty app closes. Its `--nested-desktop
--pointer-stress --interaction-delay --timeout 240` variant adds recursive dirty
close and late clean answers across a storage stall, with fresh/cold boots.
The headless `--matrix-test --release` checks malformed and stale clean replies
through nested IPC. `--matrix-storage-test --release` checks control failure,
close/exit races and replacement against the same stock program.
