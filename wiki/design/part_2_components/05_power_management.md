# Chapter 05: Power Management

> On every device with a battery, energy is a budget and wakefulness is authority. This chapter owns sleep, wake, background work, energy attribution, and the shared thermal/power envelope.

## The Legacy Model

Power management on legacy systems is a set of side mechanisms around a scheduler that does not treat energy as a resource it governs. Wake locks are ambient strings any app can hold. Background execution is a privilege granted coarsely. Battery drain is attributed after the fact by heuristic, if at all. The device sleeps and wakes through a tangle of timers, alarms, network triggers, and vendor daemons, and the user's only real lever is to force-stop an app. The power to keep the device awake, and to run while the user is not looking, is not modeled as authority. It leaks in through timers, push notifications, and background services that nobody had to grant. Below the OS, firmware and on-die power controllers make the real clock and power decisions opaquely, and the OS is a downstream client of registers it cannot observe or veto.

## The Cathedral Model

Wakefulness and background execution are capabilities, never ambient. If a component can keep the screen or CPU awake, run while backgrounded, read energy, or hold a clock frequency, it is because it holds something that says so, with a reason and a budget the user can see and revoke:

```omega
Capability<Power::PreventSleep(duration, reason)>   // a bounded, justified wake lock
Capability<Background::Run(BackgroundPolicy)>        // run-while-backgrounded, conditional
Capability<Power::Wake(WakeSource)>                  // be woken by a network/device event
Capability<Power::MaintenanceWindow(schedule)>       // batched deferred work
Capability<Power::Meter(target, granularity)>        // confidentiality-sensitive energy read
Capability<Power::Budget(domain, joules_or_watts)>   // a per-domain energy/power budget
Capability<Power::FrequencyFloor(class)>             // a guaranteed clock floor for critical work
```

`Background::Run` is the load-bearing one, because background execution must not be ambient. A backgrounded component runs only under a policy the OS enforces (on charger, on wifi, within a maintenance window, within an energy budget), and every joule it spends is attributed to it.

### Arbitration among competing holders

Competing wake and background claims are arbitrated by lease, by scheduling class, and by shedding. A wake lock is the presence of a valid lease: the device stays awake while any held, unexpired lease exists, the holder pays the attributed energy, and the user sees the lease and can revoke it live ([[human_permission_ux]]). `realtime` versus `power_saving` is the scheduling-class gradient, where a held class beats an advisory intent and the cost is attributed either way. The shared envelope is arbitrated by class-shedding, described below.

## Enforced quiescence by deprivation of means

Quiescence is enforced by the OS, not proved by the app, and it is enforced two ways. Neither is asking the app nicely. First, a component holding no clock and no wake capability has no mechanism to schedule itself. Arming a timer or registering a wake source is a gated primitive, so without the capability there is no syscall to demand a future wakeup. Second, backgrounding a component means the OS stops dispatching its domain. Even a foreign blob that busy-loops instead of yielding is quiescent when backgrounded, because the scheduler gives it zero timeslices. The app is contained, not trusted: the OS deprives it of the means and declines to run it.

What is provable is therefore a theorem about the OS, not the app: a component holding no clock or wake capability is never dispatched. It is quantified over the proved-Omega scheduler and capability core, the part of the system that carries proofs. Because it is an OS property it holds for foreign code too, with no app-side proof required. The OS can therefore answer the question no production OS can, "is this component quiescent, and if not, who is keeping it awake", and never needs to kill or statistically police a component to trust that it is idle. Legacy systems spend enormous effort failing to approximate this property. A single parked-car ECU or aftermarket accessory drains the battery because nothing can enforce that the bus actually sleeps. A decade of mobile retrofitting (wake locks, then doze, then app standby, then vendors silently killing apps) exists because wakefulness was ambient. Datacenters carry a large fraction of powered-but-idle servers nobody can prove are doing nothing. On implants the wake path becomes a battery-drain attack surface.

Bounded active background work is the harder case, and it is where proof enters. A component that runs while backgrounded under `Background::Run(policy)` must be held to a declared energy, wakeup, and CPU bound, and the code class decides how. Proved-Omega code carries the bound as a certificate, a bounded handler with a worst-case energy-and-time proof, and runs without reactive policing. This is the proven tier, and it is where the OS's own background work lives. Foreign or unproved code runs under reactive metering with a hard cutoff: the scheduler measures the proxies and suspends on overspend. That is contained and after the fact, with no silent trust, but it is not a proof. Quiescence is enforced for everyone. Staying within a bound while active is proven for native code and reactively capped for foreign code, the same split as the rest of the kernel ([[kernel_architecture]]).

Two limits keep enforced quiescence from being magic:

- **Quiescent is not zero watts.** Static leakage is an irreducible floor on real silicon, a large share of total power at advanced nodes, and a powered-but-idle core still dissipates. "Provably quiescent" means provably not active. Approaching near-zero needs power-gating (cutting the rail), not just clock-gating (stopping the clock), and even then a floor remains. This is physics, not a design choice.
- **Power-cycling is itself a consumable.** Waking and sleeping stresses hardware through thermal-cycle fatigue and wear, so quiescing aggressively to save joules trades against device lifetime. The energy model therefore charges the quiesce/wake transition, not only the on-state. A wake costs even when the run that follows is short, which is part of why maintenance windows batch wakeups.

## Energy attribution: by proxy, and metering is authority

Per-component joules are not measurable on commodity hardware, so energy is attributed by a model over measurable proxies. There are no per-task power rails, and worst-case energy is provably intractable to bound tightly. The sharper statement is that the data often exists in firmware but is withheld by policy: per-rail energy accumulators ship on some platforms behind unexposed interfaces. A Cathedral-owned platform dissolves that policy gap. The physics gap, no per-task meter on commodity parts, remains. Energy is therefore attributed by a model over the measurable proxies the OS already governs (CPU cycles, radio-on time, GPU-on time, screen-on time, coprocessor-on time) and surfaced as the user-visible "what drained my battery" ([[observability_and_introspection]]). This is not a Cathedral compromise. Every power-constrained domain independently arrived at the same posture. Capabilities improve on the commodity version by making the proxy boundaries visible and the attribution trustworthy rather than reverse-engineered after the fact.

Two rules keep the proxies accurate:

- **Charge the trigger, not the bystander (tail energy).** A radio tail, a GPS fix, or a flash write stays high-power after the code that caused it returns; for a radio, the tail is the majority of the energy. The cost is charged to the capability that triggered the high-power state, not to whatever happens to be running during the tail.
- **Offloaded work is not free, it is elsewhere.** Work pushed to a sensor hub, DSP, or accelerator is often what dominates standby draw, and it is invisible to a CPU-cycle proxy. A separate coprocessor-on proxy attributes it. The offload that saves energy must not vanish from the accounting.

Metering is a governed, confidentiality-sensitive capability, not an ambient number. Reading fine-grained energy or power is a side channel into what a component is computing. Unprivileged energy reads have been turned into cryptographic-key and address-layout leaks, and frequency scaling that tracks data-dependent power has been turned into a remote timing attack against otherwise constant-time code. `Power::Meter` is therefore non-ambient and attenuates by coarsening. A component meters itself, or what it holds authority over, at fine grain. Broadly shared readings are coarse, aggregated, and delayed (per-component totals, the safe battery-screen surface), never per-operation. This is better than the commodity world's all-or-nothing response, which removed the interface that green-power tooling needs, because the read is a capability with a gradient. What a capability check does not close is the physical channel: a confined-but-untrusted neighbor inducing voltage droop on a shared power rail to spy on or crash the host. That needs hardware power-domain isolation ([[security_policy_and_sandboxing]]), because confinement is not isolation when the substrate is shared.

## The power envelope: a shared commons with four edges

Thermal headroom is not a single ceiling. It is a **shared power envelope** the OS governs as a commons, with four edges:

- **A sustained ceiling.** The maximum power or temperature the package or chassis can hold.
- **A rate limit.** How fast load may swing (dP/dt). Large synchronized swings damage power delivery and, at scale, the grid, so transitions are slew-bounded, and a component can declare its power profile ahead of a surge so the OS shapes the ramp rather than absorbing a step.
- **A floor.** Sometimes the binding constraint is a minimum draw, for a component that must spend energy to stay alive or warm, so the envelope has a bottom as well as a top.
- **An enclosure cap.** For handhelds and wearables the limit is the temperature a human-touchable surface may reach, a low single-digit-watt budget, distinct from and below the silicon's junction limit.

Allocation across the envelope uses the same scheduling-class gradient as CPU ([[scheduler_and_resources]]). Under pressure the OS sheds best-effort and `power_saving` work first and protects foreground and real-time, with the heat and power attributed to the component that caused it. Two properties make the commons real rather than nominal:

- **Heat is an interference channel, not only a budget.** One component's heat raises a neighbor's temperature, which changes the neighbor's timing, since silicon runs slower hot. Thermal coupling is a cross-component interference the scheduler must account for, not merely a heat budget to divide. An innocent high-priority task can miss a deadline because of an unrelated hot neighbor.
- **Shedding must de-energize.** A shed that only de-schedules a component does not lower power if the device keeps running. The shed capability must actually idle or gate the underlying device.

## The frequency floor: real-time and constant-time on one primitive

The most-wished-for missing primitive across safety-critical and real-time computing is a **guaranteed clock floor**: critical work reserves a frequency it keeps while everything else scales down. Without it the field gives up. Dynamic frequency scaling is disabled in certified avionics, because a frequency change invalidates worst-case timing analysis, and general-purpose schedulers run all real-time tasks pinned at maximum frequency, burning worst-case power continuously. Cathedral makes the floor a reservation: `Power::FrequencyFloor` is a held guarantee that composes with the [[scheduler_and_resources]] gradient and the worst-case-execution-time story.

It pays a second debt. A fixed frequency also closes the frequency side channel: when the clock cannot track data-dependent power, the remote timing attack on constant-time code disappears. One reservation serves both real-time determinism and constant-time security.

Three limits apply:

- **On commodity silicon the floor cannot be guaranteed.** Firmware and the on-die power controller can legitimately undershoot a requested frequency, and an autonomous thermal trip overrides everything. The reservation degrades to best-effort, and the design says so. Only a Cathedral-owned ISA and firmware, with an honored floor and no invisible override, make it a hard contract.
- **A slower clock does not mean proportionally more time for memory-bound code.** Memory latency does not scale with the core clock, so the reservation composes with a frequency-invariant cycle bound, not a naive scaling of wall-clock timing.
- **Slowing down is not always the win.** For leaky silicon, race-to-idle then deep-sleep can beat holding a low clock, so the floor composes with the quiescence model rather than assuming a slowdown always saves energy.

## Energy as a budget per domain

Energy is a budgetable, isolatable resource per protection domain: a per-component, per-partition, per-tenant joule-or-watt budget the scheduler enforces and sheds against, entering the same authority graph as every other capability. This answers a live industry gap. Software-defined-vehicle containers, cloud tenants, partitioned avionics, and network slices all isolate CPU but have no per-domain energy isolation, and bolt power apportionment on by hand. The caveat from above applies. A budget bounds a domain's accounted spend; it does not by itself stop a shared-rail physical droop or a power-hammering denial of service, which needs hardware power-domain isolation.

Budgets and limits sit on different axes. The measurable proxies are spent budgets, checked on the scheduler's budget-check path. Wake, meter, and frequency floor are held ceilings. The per-domain `Power::Budget` is an enforced allocation the scheduler sheds against.

## Boundaries: what a capability OS does not fix

Power management has a hard floor the OS cannot cross, and naming it is part of the design:

- **Below-SoC physics.** Battery state-of-charge and state-of-health estimation drift, balancing-resistor heat, power-amplifier inefficiency, the RF idle floor, and cell chemistry are electrochemistry and analog electronics below the OS. A capability model does not improve them.
- **Commodity firmware keeps the steering wheel.** On stock parts the platform power controllers, management engine, embedded controller, and battery gas-gauge make the final power and clock decisions below any OS. Cathedral inherits every silent override and cannot honor a floor or prove zero watts. The Smart Battery specification even places authority above the host: the OS is the battery pack's child. This is consistent with the sandbox stance ([[security_policy_and_sandboxing]]), where a host owns its children's reality; here the platform is our host. Only an owned platform reclaims the wheel.
- **Fleet and grid scale live above the node.** Megawatt oversubscription, grid resonance from synchronized datacenter loads, facility cooling, and cross-node job-power budgets are above a single-node OS. Cathedral's contribution is to define how per-node power capabilities compose into a facility authority ([[distributed_boundary]]), not to solve grid physics.
- **Intermittent and batteryless operation** needs a forward-progress and checkpoint-atomicity story, surviving arbitrary power loss without livelock or corrupt state, that lives with the error and persistence model ([[error_model_and_recovery]], [[memory_and_persistence]]), plus a power-failure-survivable clock so freshness and deadlines stay defined across an outage.

## Concerns & Design Space

- **Sleep states & device power states.** The system and each device move through power states. Transitions are scheduling decisions, modeled as a `state` graph, gated by who holds wake authority.
- **Wake locks as bounded capabilities.** `PreventSleep` carries a duration and a reason, expires by lease ([[capability_lifecycle]]), and shows up in the authority graph, so there are no infinite, anonymous locks. The `reason` is legibility and audit only, never trusted. The OS cannot verify it, so enforcement is the lease bound, the attribution, and revocation. An advisory layer flags an implausible reason (a calculator claiming a media wake lock), and no component's reason is privileged.
- **Maintenance windows & wake coalescing.** The OS batches deferred work across components into shared wake windows to amortize the fixed cost of waking at all. Coalescing is opportunistic (flush deferred work when the device is already awake) plus declared windows. Predictive coalescing that models user behavior carries a privacy cost and is opt-in, not core.
- **Network wake & activation triggers.** Being woken by an inbound packet or push is a capability (`Power::Wake`), so the set of things that can rouse the device is enumerable ([[networking]]). A demand-activation trigger ([[service_activation]]) that would rouse a sleeping device is gated by `Power::Wake` and defers into a maintenance window, so wake-and-spawn stays bounded.
- **Crash/consistency under power loss.** Sudden power loss is a failure cause ([[error_model_and_recovery]]); durable state must survive it ([[memory_and_persistence]]).

## Key Questions

- Is the frequency floor's best-effort degradation on commodity silicon acceptable for the real-time classes Cathedral targets, or does hard real-time gate on owned hardware?
- Can proxy-based attribution stay accurate enough under noisy-neighbor and NUMA effects to drive capability decisions, or only to inform the user?

## Omega Leverage

- Wakefulness, background, metering, budgets, and frequency floors are capabilities (values plus domains) in the same authority graph as every other power ([Omega Chapter 19: Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Leasing ([[capability_lifecycle]]) makes bounded, expiring wake locks the default rather than the exception.
- A component with no clock or wake capability has no means to schedule itself, and the OS provably never dispatches it when backgrounded. Quiescence is an enforced OS property, not an app-carried proof ([[time_and_clocks]]).
- Power-state transitions are an Omega `state` graph the OS schedules over.
- The frequency floor composes with the worst-case-execution-time and totality story (a frequency-invariant cycle bound), and metering is a confidentiality-sensitive capability that ties to Omega's constant-time and side-channel work.
- Omega does not model energy quantities, thermal envelopes, or device power states. These are Cathedral runtime accounting and policy over Omega values.

## Open Questions

- What is the exact node-level form of the power-profile or ramp declaration (a component announcing a coming surge so the OS shapes the slew), and how does it compose upward into a fleet or facility power authority ([[distributed_boundary]])?
- How is predictive wake coalescing's privacy cost bounded, if it is offered at all?

## Related
- [[scheduler_and_resources]] — energy, the power envelope, and the frequency floor as governed resources.
- [[capability_model]] — wakefulness, background, metering, and budgets as capabilities.
- [[security_policy_and_sandboxing]] — confinement is not power isolation; the shared-rail physical channel.
- [[observability_and_introspection]] — per-component energy attribution and the "what drained my battery" surface.
- [[time_and_clocks]] — the timer/wake substrate and the power-failure-survivable clock.
- [[error_model_and_recovery]] — power loss as a failure cause; intermittent forward progress.
- [[service_activation]] — demand-activation triggers gated by the wake capability.
- [[distributed_boundary]] — how per-node power capabilities compose into a facility authority.
