# Chapter 05: Power Management

> On every device with a battery, energy is a budget and wakefulness is authority. This chapter owns sleep, wake, background work, energy attribution, and the shared thermal/power envelope.

## The Legacy Model

Power management on legacy systems is a set of side mechanisms around a scheduler that does not treat energy as a first-class resource. Wake locks are ambient strings any app can hold; background execution is a privilege granted coarsely; battery drain is attributed after the fact by heuristic, if at all. The device sleeps and wakes through a tangle of timers, alarms, network triggers, and vendor daemons, and the user's only real lever is to force-stop an app. Crucially, the power to *keep the device awake* and to *run while the user isn't looking* is not modeled as authority — it leaks in through timers, push notifications, and background services that nobody had to explicitly grant. Below the OS, firmware and on-die power controllers make the real clock and power decisions opaquely, and the OS is a downstream client of registers it cannot observe or veto.

## The Cathedral Model

Wakefulness and background execution are **capabilities**, never ambient. If a component can keep the screen or CPU awake, run while backgrounded, read energy, or hold a clock frequency, it is because it *holds* something that says so — with a reason and a budget the user can see and revoke:

```omega
Capability<Power::PreventSleep(duration, reason)>   // a bounded, justified wake lock
Capability<Background::Run(BackgroundPolicy)>        // run-while-backgrounded, conditional
Capability<Power::Wake(WakeSource)>                  // be woken by a network/device event
Capability<Power::MaintenanceWindow(schedule)>       // batched deferred work
Capability<Power::Meter(target, granularity)>        // confidentiality-sensitive energy read
Capability<Power::Budget(domain, joules_or_watts)>   // a per-domain energy/power budget
Capability<Power::FrequencyFloor(class)>             // a guaranteed clock floor for critical work
```

`Background::Run` is the load-bearing one: background execution must **not be ambient**. A backgrounded component runs only under a policy the OS enforces — on charger, on wifi, within a maintenance window, within an energy budget — and every joule it spends is attributed to it.

## Enforced quiescence — by deprivation of means

Quiescence is not something an app *proves about itself*; it is **enforced**, two ways, and neither is "ask the app nicely." First, **a component holding no clock and no wake capability has no mechanism to schedule itself** — arming a timer or registering a wake source is a gated primitive, so absent the capability there is no syscall to demand a future wakeup. Second, **backgrounding a component means the OS stops dispatching its domain** — even a foreign blob that busy-loops instead of yielding is quiescent when backgrounded, because the scheduler gives it zero timeslices. The app is *contained, not trusted*: the OS deprives it of the means and declines to run it.

What is *provable* here is therefore a theorem about the **OS**, not the app — "a component holding no clock/wake capability is never dispatched" — quantified over the proved-Omega scheduler and capability core, the part of the system that actually carries proofs. Because it is an OS property it holds for **foreign code too**, with no app-side proof required. So the OS can answer the question no production OS can — "is this component quiescent, and if not, who is keeping it awake" — and never needs to kill or statistically police a component to trust that it is idle. This is the property legacy systems spend enormous effort failing to approximate: a single parked-car ECU or aftermarket accessory drains the battery because nothing can enforce that the bus actually sleeps; a decade of mobile retrofitting (wake locks, then doze, then app-standby, then vendors silently killing apps) exists because wakefulness was ambient; datacenters carry a large fraction of powered-but-idle servers nobody can prove are doing nothing; and on implants the wake path becomes a battery-drain attack surface.

**Bounded *active* background work is the harder case, and it is where proof enters.** A component that genuinely runs while backgrounded — `Background::Run(policy)` — must be held to a declared energy/wakeup/CPU bound, and the code class decides how. **Proved-Omega code carries the bound as a certificate** (a bounded handler with a worst-case energy-and-time proof) and runs *without* reactive policing — the genuine proven tier, and where the OS's own background work lives. **Foreign or un-proved code runs under reactive metering with a hard cutoff** — the scheduler measures the proxies and suspends on overspend: contained, after-the-fact, no silent trust, but not a proof. So quiescence is enforced for everyone; staying within a bound *while active* is proven for native and reactively capped for foreign — the same split as the rest of the kernel ([[kernel_architecture]]).

Two honest limits keep even enforced quiescence from being magic:

- **Quiescent is not zero-watts.** Static leakage is an irreducible floor on real silicon — a large share of total power at advanced nodes, and a powered-but-idle core still dissipates. "Provably quiescent" means provably-not-*active*; approaching near-zero needs **power-gating** (cutting the rail), not just clock-gating (stopping the clock), and even then a floor remains. This is physics, not a design choice.
- **Power-cycling is itself a consumable.** Waking and sleeping stresses hardware (thermal-cycle fatigue, wear), so quiescing aggressively to save joules trades against device lifetime. The energy model therefore charges the **quiesce/wake transition**, not only the on-state — a wake costs even when the run that follows is short, which is part of why maintenance windows batch wakeups in the first place.

## Energy attribution: by proxy, and metering is authority

Per-component joules are **not measurable** on commodity hardware — there are no per-task power rails, and worst-case energy is provably intractable to bound tightly. The sharper statement: the data often *exists* in firmware but is **withheld by policy** (per-rail energy accumulators ship on some platforms behind unexposed interfaces). A Cathedral-owned platform dissolves that *policy* gap; the *physics* gap — no per-task meter on commodity parts — remains. So energy is **attributed** by a model over the **measurable proxies** the OS already governs — CPU cycles, radio-on time, GPU-on time, screen-on time, coprocessor-on time — and surfaced as the user-visible "what drained my battery" ([[observability_and_introspection]]). This is not a Cathedral compromise; it is the posture every power-constrained domain independently arrived at. Capabilities make it *better* than the commodity version by making the proxy boundaries explicit and the attribution trustworthy rather than reverse-engineered after the fact.

Two rules keep the proxies honest:

- **Charge the trigger, not the bystander (tail energy).** A radio tail, a GPS fix, or a flash write stays high-power *after* the code that caused it returns — for a radio, the tail is the majority of the energy. So the cost is charged to the **capability that triggered the high-power state**, not to whatever happens to be running during the tail.
- **Offloaded work is not free, it is elsewhere.** Work pushed to a sensor hub, DSP, or accelerator is often what dominates standby draw, and it is invisible to a CPU-cycle proxy. An explicit **coprocessor-on proxy** attributes it; the offload that saves energy must not vanish from the accounting.

**Metering is a governed, confidentiality-sensitive capability — not an ambient number.** Reading fine-grained energy or power is a side channel into what a component is *computing*: unprivileged energy reads have been turned into cryptographic-key and address-layout leaks, and frequency-scaling that tracks data-dependent power has been turned into a *remote* timing attack against otherwise-constant-time code. So `Power::Meter` is non-ambient and attenuates by **coarsening**: a component meters itself, or what it holds authority over, at fine grain; broadly-shared readings are **coarse, aggregated, and delayed** (per-component totals — the safe battery-screen surface), never per-operation. This is strictly better than the commodity world's all-or-nothing response (which simply removed the interface that green-power tooling needs), because the read is a capability with a gradient. What a capability check does **not** close is the *physical* channel — a confined-but-untrusted neighbor inducing voltage droop on a shared power rail to spy on or crash the host. That needs hardware power-domain isolation ([[security_policy_and_sandboxing]]): confinement is not isolation when the substrate is shared.

## The power envelope: a shared commons with four edges

Thermal headroom is not a single ceiling. It is a **shared power envelope** the OS governs as a commons, with four edges:

- a **sustained ceiling** — the maximum power/temperature the package or chassis can hold;
- a **rate limit** — how fast load may swing (dP/dt). Large synchronized swings damage power delivery and, at scale, the grid, so transitions are **slew-bounded**, and a component can **declare its power profile** ahead of a surge so the OS shapes the ramp rather than absorbing a step;
- a **floor** — sometimes the binding constraint is a *minimum* draw (a component that must spend energy to stay alive or warm), so the envelope has a bottom, not only a top;
- an **enclosure cap** — for handhelds and wearables the limit is the temperature a *human-touchable surface* may reach (a low single-digit-watt budget), distinct from and below the silicon's junction limit.

Allocation across the envelope is by the same **scheduling-class gradient** as CPU ([[scheduler_and_resources]]): under pressure the OS sheds best-effort and `power_saving` work first and protects foreground and real-time, with the **heat/power attributed** to the component that caused it. Two properties make the commons real rather than nominal:

- **Heat is an interference channel, not only a budget.** One component's heat raises a neighbor's temperature, which changes the neighbor's timing (silicon runs slower hot). So thermal coupling is a cross-component *interference* the scheduler must account for — an innocent high-priority task can miss a deadline because of an unrelated hot neighbor — not merely a heat budget to divide.
- **Shedding must de-energize.** A "shed" that only de-schedules a component does not lower power if the device keeps running; the shed capability must actually idle or gate the underlying device.

## The frequency floor: real-time and constant-time on one primitive

The most-wished-for missing primitive across safety-critical and real-time computing is a **guaranteed clock floor**: let critical work reserve a frequency it keeps while everything else scales down. Without it the field gives up — dynamic frequency scaling is *disabled* in certified avionics (a frequency change invalidates worst-case timing analysis), and general-purpose schedulers run *all* real-time tasks pinned at maximum frequency, burning worst-case power continuously. Cathedral makes the floor a first-class reservation: `Power::FrequencyFloor` is a held guarantee composing with the [[scheduler_and_resources]] gradient and the worst-case-execution-time story.

It pays a second debt: a fixed frequency also **closes the frequency side channel** — when the clock cannot track data-dependent power, the remote timing attack on constant-time code disappears. One reservation serves both real-time determinism and constant-time security.

Honest limits, stated rather than hidden:

- On **commodity** silicon the floor cannot be *guaranteed* — firmware and the on-die power controller can legitimately undershoot a requested frequency, and an autonomous thermal trip overrides everything. The reservation degrades to best-effort, and the design says so. Only a **Cathedral-owned ISA/firmware** (an honored floor, no invisible override) makes it a hard contract.
- The "slower clock means proportionally more time" assumption is **false for memory-bound code** (memory latency does not scale with the core clock), so the reservation composes with a **frequency-invariant cycle bound**, not a naive scaling of wall-clock timing.
- Slowing down is not always the win: for leaky silicon, **race-to-idle then deep-sleep** can beat holding a low clock, so the floor composes with the quiescence model rather than assuming a slowdown always saves energy.

## Energy as a first-class budget per domain

Energy is a **budgetable, isolatable resource per protection domain**: a per-component, per-partition, per-tenant joule-or-watt budget the scheduler enforces and sheds against, entering the same authority graph as every other capability. This is the directly applicable answer to a live industry gap — software-defined-vehicle containers, cloud tenants, partitioned avionics, and network slices all isolate CPU but have **no** per-domain energy isolation, and bolt power apportionment on by hand. With the caveat from above: a *budget* bounds a domain's accounted spend; it does **not** by itself stop a shared-rail physical droop or a power-hammering denial of service, which needs hardware power-domain isolation.

## Honest boundaries (what a capability OS does not fix)

Power management has a hard floor the OS cannot cross, and naming it is part of the design:

- **Below-SoC physics.** Battery state-of-charge and state-of-health estimation drift, balancing-resistor heat, power-amplifier inefficiency, the RF idle floor, cell chemistry — electrochemistry and analog electronics below the OS. A capability model does not improve them.
- **Commodity firmware keeps the steering wheel.** On stock parts the platform power controllers, management engine, embedded controller, and battery gas-gauge make the final power and clock decisions below any OS — Cathedral inherits every silent override and cannot honor a floor or prove zero-watts. The Smart Battery specification even places authority *above* the host: the OS is the battery pack's child. This is consistent with the sandbox stance ([[security_policy_and_sandboxing]]) — a host owns its children's reality, and here the platform is our host. Only an owned platform reclaims the wheel.
- **Fleet and grid scale live above the node.** Megawatt oversubscription, grid resonance from synchronized datacenter loads, facility cooling, and cross-node job-power budgets are above a single-node OS. Cathedral's contribution is to define how per-node power capabilities **compose** into a facility authority ([[distributed_boundary]]), not to solve grid physics.
- **Intermittent and batteryless operation** needs a forward-progress and checkpoint-atomicity story — surviving arbitrary power loss without livelock or corrupt state — that lives with the error and persistence model ([[error_model_and_recovery]], [[memory_and_persistence]]), plus a **power-failure-survivable clock** so freshness and deadlines stay defined across an outage.

## Concerns & Design Space

- **Sleep states & device power states.** The system and each device move through power states; transitions are scheduling decisions, modeled as a `state` graph, gated by who holds wake authority.
- **Wake locks as bounded capabilities.** `PreventSleep` carries a duration *and* a reason, expires by lease ([[capability_lifecycle]]), and shows up in the authority graph — no infinite, anonymous locks. The `reason` is **legibility and audit only, never trusted**: the OS cannot verify it, so enforcement is the lease bound, the attribution, and revocation; an advisory layer flags an implausible reason (a calculator claiming a media wake lock), and no component's reason is privileged.
- **Maintenance windows & wake coalescing.** The OS batches deferred work across components into shared wake windows to amortize the fixed cost of waking at all. Coalescing is opportunistic (flush deferred work when the device is already awake) plus explicit windows; *predictive* coalescing that models user behavior carries a privacy cost and is opt-in, not core.
- **Network wake & activation triggers.** Being woken by an inbound packet/push is a capability (`Power::Wake`), so the set of things that can rouse the device is enumerable ([[networking]]). A demand-activation trigger ([[service_activation]]) that would rouse a sleeping device is gated by `Power::Wake` and defers into a maintenance window, so wake-and-spawn stays bounded.
- **Crash/consistency under power loss.** Sudden power loss is a failure cause ([[error_model_and_recovery]]); durable state must survive it ([[memory_and_persistence]]).

## Key Questions

- **Default for a component with no power capabilities — resolved:** enforced-quiescent when backgrounded — no means to self-schedule *and* the OS does not dispatch it (an OS theorem, not an app proof; holds for foreign code), modulo the leakage floor and transition cost noted above.
- **Arbitration of competing wake/background — resolved:** a wake lock is presence-of-a-valid-lease (the device stays awake while any held, unexpired lease exists), the holder pays the attributed energy, and the user sees and revokes live ([[human_permission_ux]]); `realtime` versus `power_saving` is the scheduling-class gradient (a held class beats an advisory intent, cost attributed); the shared envelope is arbitrated by class-shedding.
- **Budget versus limit — resolved (both, on different axes):** the measurable proxies are *spent budgets* (the scheduler's budget-check path); wake, meter, and frequency-floor are *held ceilings*; and the per-domain `Power::Budget` is an enforced, shed-against allocation.
- **Who asserts a wake-lock reason — resolved:** self-asserted, untrusted, legibility/audit only; the teeth are lease, attribution, and revocation, with the advisory layer surfacing anomalies.

## Omega Leverage

- Wakefulness, background, metering, budgets, and frequency floors are **capabilities (values + domains)** in the same authority graph as every other power ([../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- **Leasing** ([[capability_lifecycle]]) makes bounded, expiring wake locks the default rather than the exception.
- A component with no clock/wake capability has no means to schedule itself, and the OS **provably never dispatches** it when backgrounded — quiescence is an enforced OS property, not an app-carried proof ([[time_and_clocks]]).
- Power-state transitions are an Omega **`state` graph** the OS schedules over.
- The frequency floor composes with the worst-case-execution-time / totality story (a frequency-invariant cycle bound), and metering is a confidentiality-sensitive capability that ties to Omega's constant-time / side-channel work.
- Omega does **not** model energy quantities, thermal envelopes, or device power states — these are Cathedral runtime accounting and policy over Omega values.

## Open Questions

- What is the exact node-level form of the **power-profile / ramp declaration** (a component announcing a coming surge so the OS shapes the slew), and how does it compose upward into a fleet/facility power authority ([[distributed_boundary]])?
- Is the frequency floor's **best-effort degradation on commodity silicon** acceptable for the real-time classes Cathedral targets, or does hard real-time gate on owned hardware?
- Can proxy-based attribution stay accurate enough under noisy-neighbor and NUMA effects to drive *capability* decisions, or only to inform the user?
- How is predictive wake coalescing's **privacy cost** bounded if it is offered at all?

## Related
- [[scheduler_and_resources]] — energy, the power envelope, and the frequency floor as governed resources.
- [[capability_model]] — wakefulness, background, metering, and budgets as capabilities.
- [[security_policy_and_sandboxing]] — confinement is not power isolation; the shared-rail physical channel.
- [[observability_and_introspection]] — per-component energy attribution and the "what drained my battery" surface.
- [[time_and_clocks]] — the timer/wake substrate and the power-failure-survivable clock.
- [[error_model_and_recovery]] — power loss as a failure cause; intermittent forward progress.
- [[service_activation]] — demand-activation triggers gated by the wake capability.
- [[distributed_boundary]] — how per-node power capabilities compose into a facility authority.
