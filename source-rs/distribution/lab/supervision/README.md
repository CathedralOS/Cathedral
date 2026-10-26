# Supervision proof

Start at [main.rs](main.rs) for role selection. The x86 smoke boot composes these
roles in [the supervision lab](../../../kernel/boot/uefi/lab/supervision/supervision.rs).
Boot supplies authority and checks task outcomes and exact heap/frame reclamation;
the userspace roles decide when to launch, communicate, cancel and collect.

The [restart entrance](restarts/restarts.rs) publishes a shared command, waits for
the child fault, then collects the exact outstanding client reply. Its
[client](restarts/client.rs) connects to each of 32 service incarnations, checks
four exact echoes, requests a deliberate fault and observes terminal closure.
Each failed connection must return its private-page, reader-page, channel and
notification usage to the baseline before acknowledging completion. Stolen
launch/child names and stale endpoint names remain explicit denial probes.
The [service](restarts/service.rs) starts with fresh process state every time.

[One-shot gates](gates.rs) coordinate completion in [failure probes](failures.rs)
and watchdog completion/idle probes, and ready/start/done in [CPU tests](cpu.rs).
They signal before exporting a wait-only right through typed endpoint escrow.
There is no copied application payload, shared page or clock requirement. The
queued/accepted right retains funding even if the sender immediately exits; the
receiver consumes and releases it. This is a fixture gate, not a general message
protocol or a claim that notification receipt proves application completion.

[Sibling tests](multiple.rs) retain one shared connection per child. A sibling's
outstanding reply remains unconsumed while its neighbour is cancelled, reaped and
replaced, then completes without replay. Failed admission must leave the healthy
connection usable. Keyboard readiness uses a gate so the raw input waiter keeps
its clock-denied and clock-granted variants and exact timeout checks.

The [link graph](links/links.rs) retains four owner connections; its application
uses three shared provider connections, while each provider serves independent
owner and application streams. Sixteen replacements alternate the application
and its providers. [Name probes](links/names.rs) check stable unrelated clock/link
names and rejection of previous incarnations. The first non-probe command remains
the explicit all-providers-live barrier; starting the app does not imply that its
providers have already been admitted.

These owners deliberately leave their final clients to process teardown instead
of Rust drop. Closing them early would let providers observe disconnection before
the parent-death cancellation this fixture is meant to test. Replacement clients
are explicitly retired after child collection and before reconnecting. Boot funds
the exact stream counts and checks all surviving children were cancelled and all
heap/frame resources reclaimed.

The failed-admission self-loop deliberately tests that proposed endpoint grants
remain usable after rejection. It still uses raw copied sends/receives for that
custody probe. Application traffic remains copied in the watchdog recovery
exchange. That is subsequent migration work; preserve denied-clock roles instead
of deleting their assertions to accommodate a new transport. Raw stale-name
send probes in the migrated sibling/graph tests are intentional authority checks.

Run both release and development `run.py --smoke` from the repository's
`tools/boot-harness-rs/` harness. Development runs also exercise larger images and
longer admission paths. These fixtures are x86-only and do not alter the normal
F5 desktop composition.
