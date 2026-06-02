# Chapter 12: Time, Clocks & Timers

> Time looks like plumbing and is actually authority. This chapter owns clocks, timers, and the principle that access to *which* clock is a capability.

## The Legacy Contract

A traditional OS gives every process the time for free. `gettimeofday`, `clock_gettime`, `sleep`, timers — all ambient, all available to anyone. There is usually one wall clock (which jumps when NTP corrects it or the user changes it) and one monotonic clock, and the application is left to know which is safe for what. This ambient time is quietly load-bearing for security: certificate validity, token expiry, and replay windows all trust the clock, yet any code can read it, depend on it, and be fooled by it. And because time is ambient, code is hard to test deterministically and impossible to run in virtual time without intercepting the whole runtime.

## What Cathedral Wants

Time is not one thing and not free. Cathedral distinguishes clocks and treats *access to each* as a held capability, granted deliberately:

```omega
Capability<Clock::Monotonic>          // duration, never jumps
Capability<Clock::Wall>               // calendar time, may jump
Capability<Clock::Trusted>            // attested, for cert/replay decisions
Capability<Clock::Virtual(timeline)>  // simulation / test time
Capability<Timer::Schedule(deadline)> // wakeups, gated by power policy
```

The consequences are deliberate. A deterministic component gets *no* wall-clock access, so it cannot accidentally become non-reproducible. A test runner gets a **virtual** clock and drives time by hand. A security-sensitive protocol demands a **trusted** clock and refuses to validate certificates against an attacker-influenced one ([[07_secrets_and_keys]]). Reading a clock is an **effect** (`clock_read`), so the authority graph shows exactly which components depend on time and which clock.

## Concerns & Design Space

- **Clock taxonomy.** Wall vs. monotonic vs. trusted vs. virtual — each a separate capability with separate trust and separate failure modes.
- **Time as authority for security.** Cert validation, token/lease expiry ([[04_capability_lifecycle]]), and replay prevention must bind to a *trusted* clock, not an ambient one ([[07_secrets_and_keys]]).
- **Timers, sleep & wake.** A scheduled wakeup is a power event, not a free operation — it must be gated by background/power policy ([[14_power_management]], [[10_scheduler_and_resources]]).
- **Scheduling deadlines.** The scheduler's deadlines ([[10_scheduler_and_resources]]) and a component's timers draw on the same time substrate.
- **Virtual time & deterministic simulation.** Granting a virtual clock lets the test/sim harness control the entire timeline, enabling deterministic replay and model checking ([[40_testing_and_simulation]]).
- **Distributed causality.** Across the [[17_distributed_boundary]] there is no single now; causality (logical/hybrid clocks) matters more than wall time, and lease expiry under partition is genuinely hard.
- **Clock drift, jumps & corrections.** Components must declare whether they tolerate a wall-clock jump; the OS should never silently hand jumpy time to code that assumed monotonicity.

## Key Questions

- What is the minimal default — does an ordinary component get *any* clock without asking?
- Who is trusted to mint a `Clock::Trusted`, and what attests it (secure element, network time authority, boot measurement [[25_boot_and_trust_chain]])?
- How does virtual time compose when a component under test calls a real service that holds a real clock?
- What is the correct lease-expiry semantics when the only available clock is unreliable or partitioned?

## Omega Leverage

- Clock access is a **capability** plus the **`clock_read` effect** — both already in Omega's vocabulary, so time appears in the authority graph and effect ceiling with no new machinery ([../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- A component with no clock capability is **statically** provable to be time-independent — the foundation of deterministic simulation.
- **Virtual time** is just a different `BoundaryProvider` behind `Clock::Read`; the test harness swaps the provider, the component never knows ([[40_testing_and_simulation]]).
- Omega does **not** model the *trust level* of a clock or distributed causality semantics — Cathedral adds the clock taxonomy and the trusted-time attestation as runtime/provider structure.

## Open Questions

- Is "trusted time" achievable on commodity hardware without a secure time source, or is it always best-effort with attestation?
- Should monotonic time be the universal default and wall-clock the privileged exception, inverting the legacy convention?
- How much of distributed time belongs here vs. in [[17_distributed_boundary]]?

## Related
- [[03_capability_model]] — clocks are capabilities in the authority graph.
- [[07_secrets_and_keys]] — trusted time underpins cert/replay decisions.
- [[17_distributed_boundary]] — causality and lease expiry without a global now.
- [[40_testing_and_simulation]] — virtual time as the basis of deterministic tests.
