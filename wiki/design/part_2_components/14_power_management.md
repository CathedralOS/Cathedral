# Chapter 14: Power Management

> On every device with a battery, energy is a budget and wakefulness is authority. This chapter owns sleep, wake, background work, and energy attribution.

## The Legacy Contract

Power management on legacy systems is a pile of side channels around a scheduler that doesn't really believe in energy. Wake locks are ambient strings any app can hold; background execution is a privilege granted coarsely and abused widely; battery drain is attributed after the fact by heuristic, if at all. The device sleeps and wakes through a tangle of timers, alarms, network triggers, and vendor daemons, and the user's only real lever is to force-stop an app. Crucially, the power to *keep the device awake* and to *run while the user isn't looking* is not modeled as authority — it leaks in through timers, push notifications, and background services that nobody had to explicitly grant.

## What Cathedral Wants

Wakefulness and background execution are **capabilities**, never ambient. If a component can keep the screen or CPU awake, or run while backgrounded, it is because it *holds* something that says so — with a reason and a budget the user can see and revoke:

```omega
Capability<Power::PreventSleep(duration, reason)>   // a bounded, justified wake lock
Capability<Background::Run(BackgroundPolicy)>        // run-while-backgrounded, conditional
Capability<Power::Wake(WakeSource)>                  // be woken by a network/device event
Capability<Power::MaintenanceWindow(schedule)>       // batched deferred work
```

`Background::Run` is the load-bearing one: background execution must **not be ambient**. A backgrounded component runs only under a policy the OS enforces — on charger, on wifi, within a maintenance window, within an energy budget — and every joule it spends is attributed to it ([[33_observability_and_introspection]]).

## Concerns & Design Space

- **Sleep states & device power states.** The system and each device move through power states; transitions are scheduling decisions, modeled as a `state` graph, gated by who holds wake authority.
- **Wake locks as bounded capabilities.** `PreventSleep` carries a duration *and* a reason, expires by lease ([[04_capability_lifecycle]]), and shows up in the authority graph — no infinite, anonymous locks.
- **Background work under policy.** Deferred sync, prefetch, and maintenance run in batched windows under power/network conditions ([[10_scheduler_and_resources]], [[12_time_and_clocks]] for the timer/wake substrate).
- **Battery & energy attribution.** Per-component energy accounting feeds both the resource governor ([[10_scheduler_and_resources]]) and a user-visible "what drained my battery" surface ([[33_observability_and_introspection]]).
- **Thermal policy.** Thermal headroom is a shared budget; throttling is a scheduling decision, and a component can declare `power_saving` intent ([[10_scheduler_and_resources]]).
- **Network wake.** Being woken by an inbound packet/push is a capability (`Power::Wake`), so the set of things that can rouse the device is enumerable ([[16_networking]]).
- **Maintenance windows.** The OS batches deferred work across components into shared wake windows to amortize the cost of waking at all.
- **Crash/consistency under power loss.** Sudden power loss is a failure cause ([[13_error_model_and_recovery]]); durable state must survive it ([[11_memory_and_persistence]]).

## Key Questions

- What is the default for a component with no power capabilities — fully suspended when backgrounded, with zero wakeups?
- How are competing wake locks and background requests arbitrated, and can the user always see and revoke them live ([[28_human_permission_ux]])?
- Is energy a *budget* a component spends (like CPU), a *limit* it must stay under, or both?
- Who is trusted to assert a justified `reason` for a wake lock, and is the reason user-auditable?

## Omega Leverage

- Wakefulness and background rights as **capabilities (values + domains)** — they enter the same authority graph as every other power ([../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- **Leasing** ([[04_capability_lifecycle]]) makes bounded, expiring wake locks the default rather than the exception.
- A component with no clock/wake capability is **provably** quiescent when backgrounded — it *cannot* schedule itself to run ([[12_time_and_clocks]]).
- Power-state transitions are an Omega **`state` graph** the OS schedules over.
- Omega does **not** model energy quantities, thermal budgets, or device power states — these are Cathedral runtime accounting and policy over Omega values.

## Open Questions

- Can energy attribution be accurate enough on commodity hardware to drive capability decisions, or only to inform the user?
- How does background policy interact with realtime/latency-sensitive intent — who wins when `realtime_audio` meets `power_saving`?
- Should the OS predictively coalesce wakeups across components, and how is that fairness arbitrated across tenants?

## Related
- [[03_capability_model]] — wakefulness and background as capabilities.
- [[10_scheduler_and_resources]] — energy and thermal as governed resources.
- [[09_component_model]] — the instance as the unit of background execution.
- [[33_observability_and_introspection]] — per-component battery attribution.
