# Chapter 03: Time, Clocks & Timers

> Reading a clock is a granted authority, and which clock a component holds matters. This chapter owns clocks, timers, and the rule that access to each kind of clock is a capability.

## The Legacy Model

A traditional OS gives every process the time for free. `gettimeofday`, `clock_gettime`, `sleep`, and timers are all ambient and available to anyone. There is usually one wall clock, which jumps when NTP corrects it or the user changes it, and one monotonic clock, and the application is left to know which is safe for what. This ambient time is quietly load-bearing for security. Certificate validity, token expiry, and replay windows all trust the clock, yet any code can read it, depend on it, and be fooled by it. And because time is ambient, code is hard to test deterministically and impossible to run in virtual time without intercepting the whole runtime.

## The Cathedral Model

Time is not one thing and not free. Cathedral distinguishes clocks and treats access to each as a held capability:

```omega
Capability<Clock::Monotonic>          // duration, never jumps
Capability<Clock::Wall>               // calendar time, may jump
Capability<Clock::Trusted>            // attested, for cert/replay decisions
Capability<Clock::Virtual(timeline)>  // simulation / test time
Capability<Clock::Wake>               // arm a wakeup on a clock; a power event
```

A deterministic component gets no wall-clock access, so it cannot accidentally become non-reproducible. A test runner gets a **virtual clock** and drives time by hand. A security-sensitive protocol demands a **trusted clock** and refuses to validate certificates against an attacker-influenced one ([[secrets_and_keys]]). Reading a clock reaches the selected `Clock` boundary service, so the authority and effect reports show which components depend on time and which provider supplies it.

### The decided mechanism

Clock-as-capability costs nothing per read. The capability selects which time page a principal is mapped, once at grant. The read itself is the ordinary cheap userspace path: a free-running hardware counter (`rdtsc` on x86, `CNTVCT` on ARM) scaled by constants in that page, with no syscall. `Monotonic` and `Wall` stay as fast as any vDSO read. The capability gates whether a principal reads time and which time it reads, not how fast. `Clock::Trusted` is the exception. It is the rare, costlier path, because it checks attestation, and it is used only for security-expiry decisions.

Virtual-clock dilation has two tiers, because a visible scale is bypassable. A parent serves a child a dilated clock by mapping it a page with different `scale`/`offset` constants. This is cheap. The free-running counter supplies the motion, so the page is written only when the dilation changes, with no hot loop and no per-read trap. But it is cooperative only. An adversarial child reads the raw counter and measures the effective rate (`Δtime / Δcounter`), so it cannot be fooled. Obfuscating or encrypting the constants does not help, because the child measures the behavior and never needs the parameter. A linear transform extracts in two queries anyway, and you cannot keep a secret from code running on its own core.

- **Cooperative (default, fast).** Software `scale`/`offset`. It binds cooperative software: test harnesses, replay, offsets. An adversarial child can read real time, which is fine, since it is confined, and "am I in a Matrix?" is meaningless when everything is one.
- **Non-bypassable (rare).** Gate the raw counter. `CR4.TSD` on x86 or `CNTKCTL` on ARM makes a userspace counter read trap, routing all time through the host. This is correct but slow, a trap per read, so it is reserved for the rare Matrix that must not see real time: adversarial deterministic replay, or a paranoid anti-fingerprint sandbox.

No VT-x for native Matrices. Hardware-transparent offsetting (x86 VT-x TSC-offset, ARM `CNTVOFF`) would give fast non-bypassable dilation, but it needs the virtualization stack Cathedral avoids for native domains ([[kernel_architecture]]). The non-bypassable case is rare and the slow counter-gate covers it, so Matrices stay lightweight MMU-confined domains, and VT-x is reserved for the compatibility box running a real foreign OS ([[compatibility_and_legacy]]). The instruction gate (`CR4.TSD`/`CNTKCTL`) is a basic, universal CPU control in the same family as the MMU permission bits, not a virtualization dependency.

`Clock::Trusted` is composed from several sources, not read from one. The threat it defends against is rollback: setting the clock backward to un-expire a certificate, token, or lease. It is built from four parts. An **anti-rollback monotonic counter** (secure element, TPM, or TrustZone, persisted) guarantees that time can never precede the last durably recorded point; this is the load-bearing part. Provable calendar accuracy comes from Roughtime-style signed, multi-server cross-checked time, so a lying server is catchable, unlike unauthenticated NTP. Continuity between syncs comes from the free-running counter. All of it is maintained by measured-boot-attested kernel code ([[boot_and_trust_chain]]). The RTC is only the plausible cold-start baseline. Trusted time is therefore best-effort but attestable. It is never perfect, but it is rollback-resistant and provable, which is what expiry decisions need. Only the rare expiry decision needs it; `Monotonic` and `Wall` need none of this.

ARM is 1:1 with x86 here. `rdtsc` maps to `CNTVCT`/`CNTPCT`, `CR4.TSD` to `CNTKCTL`, and VT-x TSC-offset to `CNTVOFF`, which is built into the timer and if anything cleaner.

A component's time frame is its clock capability, and the frame is Matrix-scoped. Within a frame, time is consistent. Crossing out of the Matrix is the distributed case, with no global now, and it is handled in [[distributed_boundary]]. A local cross-frame mismatch arises only from a cross-frame capability the granter chose to hand out: a virtual-clock component handed a real service, or an un-owned, differently dilated child. It is never a surprise, because the granter chose it. The deterministic-test rule follows. Hand the component a virtual clock and virtual versions of its dependencies, and it stays in one coherent frame. Hand it a real capability, and that call is a real-world side effect by your own setup. The one hard residual, cross-machine causality with no global now, lives in [[distributed_boundary]].

A custom ISA dissolves all of this by construction. Bake the per-domain offset and scale into the counter-read instruction itself, behind an unreadable domain register, and every time read is transparent, controlled, non-bypassable, and fast by default, with no trap and no virtualization stack. More broadly, a capability-native ISA (the CHERI lineage) turns the whole capability check, including clock-read gating, into a cheap instruction rather than a trap or an MMU game. The cost, leak, and bypass trade-offs above are artifacts of commodity silicon that never had capabilities or per-domain time in its instruction set. On Cathedral's own ISA they are non-problems, limited only to where code actually runs on that ISA. Foreign code and the compatibility box still ride commodity hardware.

## Concerns & Design Space

- **Clock taxonomy.** Wall, monotonic, trusted, and virtual are each a separate capability with separate trust and separate failure modes.
- **Time as authority for security.** Certificate validation, token and lease expiry ([[capability_lifecycle]]), and replay prevention must bind to a trusted clock, not an ambient one ([[secrets_and_keys]]).
- **A timer is a clock-conditioned wakeup.** There is no timer object. A timer is parking a task until a clock reaches a time: the scheduler's one wait primitive ([[scheduler_and_resources]]) applied to a clock condition. Sleep, timeout, deadline, and periodic work all compose from it. Because a wakeup can rouse a sleeping device, arming one that wakes the device needs a wakefulness capability and is gated by power policy ([[power_management]]).
- **Scheduling deadlines.** The scheduler's deadlines ([[scheduler_and_resources]]) and a component's timers draw on the same time substrate.
- **Virtual time & deterministic simulation.** Granting a virtual clock lets the test or simulation harness control the entire timeline, enabling deterministic replay and adversarial scheduling tests ([[testing_and_simulation]]).
- **Nested clocks.** A virtual clock is the recursive-provider pattern ([[capability_model]]) applied to time. A parent serves a child's clock and may pause, scale, or fabricate it, which is also what a virtual machine's clock and a test harness's timeline are.
- **Distributed causality.** Across the [[distributed_boundary]] there is no single now. Causality (logical or hybrid clocks) matters more than wall time, and lease expiry under partition is hard.
- **Clock drift, jumps & corrections.** Components must declare whether they tolerate a wall-clock jump. The OS should never silently hand jumpy time to code that assumed monotonicity.
- **Zero value.** A zero clock capability is the inert null clock (shape 2). Holding no clock is the deterministic, time-independent default, and a zero `Duration` is a valid 0 (shape 1), so a zeroed timer fires immediately rather than erroring ([[omega_substrate]]).

## Key Questions

- What is the minimal default? Does an ordinary component get any clock without asking?
- What is the correct lease-expiry semantics when the only available clock is unreliable or partitioned?

## Omega Leverage

- Clock access is a capability plus reach to the `Clock` boundary-service identity. Both are already in Omega's vocabulary, so time appears in the authority graph and reach ceiling with no new machinery ([Omega Chapter 19: Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- A component with no clock capability is statically provable to be time-independent, which is the foundation of deterministic simulation.
- Virtual time is a different `BoundaryProvider` behind `Clock::Read`. The test harness swaps the provider, and the component never knows ([[testing_and_simulation]]).
- Omega does not model the trust level of a clock or distributed causality semantics. Cathedral adds the clock taxonomy and the trusted-time attestation as runtime and provider structure.

## Open Questions

- Should monotonic time be the universal default and wall-clock the privileged exception, inverting the legacy convention?
- How much of distributed time belongs here versus in [[distributed_boundary]]?

## Related
- [[capability_model]] — clocks are capabilities in the authority graph.
- [[secrets_and_keys]] — trusted time underpins cert/replay decisions.
- [[distributed_boundary]] — causality and lease expiry without a global now.
- [[testing_and_simulation]] — virtual time as the basis of deterministic tests.
