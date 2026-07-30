# Chapter 03: Time, Clocks & Timers

> Time looks like plumbing and is actually authority. This chapter owns clocks, timers, and the principle that access to *which* clock is a capability.

## The Legacy Model

A traditional OS gives every process the time for free. `gettimeofday`, `clock_gettime`, `sleep`, timers — all ambient, all available to anyone. There is usually one wall clock (which jumps when NTP corrects it or the user changes it) and one monotonic clock, and the application is left to know which is safe for what. This ambient time is quietly load-bearing for security: certificate validity, token expiry, and replay windows all trust the clock, yet any code can read it, depend on it, and be fooled by it. And because time is ambient, code is hard to test deterministically and impossible to run in virtual time without intercepting the whole runtime.

## The Cathedral Model

Time is not one thing and not free. Cathedral distinguishes clocks and treats *access to each* as a held capability, granted deliberately:

```omega
Capability<Clock::Monotonic>          // duration, never jumps
Capability<Clock::Wall>               // calendar time, may jump
Capability<Clock::Trusted>            // attested, for cert/replay decisions
Capability<Clock::Virtual(timeline)>  // simulation / test time
Capability<Clock::Wake>               // arm a wakeup on a clock; a power event
```

The consequences are deliberate. A deterministic component gets *no* wall-clock access, so it cannot accidentally become non-reproducible. A test runner gets a **virtual** clock and drives time by hand. A security-sensitive protocol demands a **trusted** clock and refuses to validate certificates against an attacker-influenced one ([[secrets_and_keys]]). Reading a clock reaches the selected `Clock` boundary service, so the authority/effect reports show exactly which components depend on time and which provider supplies it.

### The decided mechanism

**Clock-as-capability costs nothing per read.** The capability selects *which time page a principal is mapped*, once at grant; the read itself is the ordinary cheap userspace path — a free-running hardware counter (`rdtsc` / ARM `CNTVCT`) scaled by constants in that page, no syscall. `Monotonic` and `Wall` stay as fast as any vDSO read; the capability gates *whether and which*, not *how fast*. `Clock::Trusted` is the exception — the rare, costlier path (it checks attestation), used only for security-expiry decisions.

**Virtual-clock dilation has two honest tiers, because a visible scale is bypassable.** A parent serves a child a dilated clock by mapping it a page with different `scale`/`offset` constants — cheap, and the free-running counter supplies the motion, so the page is written only when the dilation *changes* (no hot loop, no per-read trap). But this is **cooperative-only**: an adversarial child reads the *raw* counter and *measures* the effective rate (`Δtime / Δcounter`), so it cannot be fooled — and obfuscating/encrypting the constants does not help, because the child measures the *behavior*, never reads the *parameter* (a linear transform extracts in two queries anyway; you cannot keep a secret from code on its own core). So:
- **Cooperative (default, fast):** software `scale`/`offset`. Binds honest software (test harnesses, replay, offsets); an adversarial child can read real time — which is **fine**, since it is confined and "am I in a Matrix?" is meaningless when *everything* is one.
- **Non-bypassable (rare):** gate the raw counter — `CR4.TSD` (x86) / `CNTKCTL` (ARM) makes a userspace counter read *trap*, routing all time through the host. Correct but **slow** (a trap per read), so it is reserved for the rare Matrix that genuinely must not see real time (adversarial deterministic replay, a paranoid anti-fingerprint sandbox).

**No VT-x for native Matrices.** Hardware-transparent offsetting (x86 VT-x TSC-offset / ARM `CNTVOFF`) *would* give fast non-bypassable dilation, but it needs the virtualization stack Cathedral deliberately avoids for native domains ([[kernel_architecture]]). Since the non-bypassable case is rare and the slow counter-gate covers it, Matrices stay lightweight MMU-confined domains; **VT-x is reserved for the compatibility box** running a real foreign OS ([[compatibility_and_legacy]]). The instruction-gate (`CR4.TSD`/`CNTKCTL`) is a basic, universal CPU control in the same family as the MMU permission bits — not a virtualization dependency.

**`Clock::Trusted` is composed, not a single source.** The threat it defends is **rollback** — setting the clock backward to un-expire a cert, token, or lease. So it is: an **anti-rollback monotonic counter** (secure element / TPM / TrustZone, persisted, so time can never precede the last durably-recorded point — the load-bearing part); **provable calendar accuracy** (Roughtime-style *signed*, multi-server-cross-checked time, so a lying server is catchable, unlike unauthenticated NTP); **continuity** from the free-running counter between syncs; all maintained by **measured-boot-attested** kernel code ([[boot_and_trust_chain]]). The RTC is only the plausible cold-start baseline. So trusted time is best-effort but *attestable* — and only the rare expiry decision needs it; `Monotonic`/`Wall` need none.

**ARM is 1:1.** `rdtsc`→`CNTVCT`/`CNTPCT`, `CR4.TSD`→`CNTKCTL`, VT-x TSC-offset→`CNTVOFF` (built into the timer, if anything cleaner). Same problems, same solutions, same decisions.

**A custom ISA dissolves all of this by construction.** Bake the per-domain offset/scale into the counter-read instruction itself, behind an unreadable domain register, and every time read is *transparent, controlled, non-bypassable, and fast* by default — no trap, no virtualization stack. More broadly, a capability-native ISA (the CHERI lineage) turns the whole capability check — including clock-read gating — into a cheap instruction rather than a trap or MMU game. The cost/leak/bypass trade-offs above are artifacts of commodity silicon that never had capabilities or per-domain time in its instruction set; on Cathedral's own ISA they are non-problems — limited only to where code actually runs on that ISA (foreign code and the compat box still ride commodity hardware).

## Concerns & Design Space

- **Clock taxonomy.** Wall vs. monotonic vs. trusted vs. virtual — each a separate capability with separate trust and separate failure modes.
- **Time as authority for security.** Cert validation, token/lease expiry ([[capability_lifecycle]]), and replay prevention must bind to a *trusted* clock, not an ambient one ([[secrets_and_keys]]).
- **A timer is a clock-conditioned wakeup.** There is no timer object: a timer is parking a task until a clock reaches a time, the scheduler's one wait primitive ([[scheduler_and_resources]]) applied to a clock condition. Sleep, timeout, deadline, and periodic work all compose from it. Because a wakeup can rouse a sleeping device, arming one that wakes the device needs a wakefulness capability and is gated by power policy ([[power_management]]).
- **Scheduling deadlines.** The scheduler's deadlines ([[scheduler_and_resources]]) and a component's timers draw on the same time substrate.
- **Virtual time & deterministic simulation.** Granting a virtual clock lets the test/sim harness control the entire timeline, enabling deterministic replay and model checking ([[testing_and_simulation]]).
- **Nested clocks.** A virtual clock is the recursive-provider pattern ([[capability_model]]) applied to time: a parent serves a child's clock and may pause, scale, or fabricate it, which is exactly a virtual machine's clock as well as a test harness's timeline.
- **Distributed causality.** Across the [[distributed_boundary]] there is no single now; causality (logical/hybrid clocks) matters more than wall time, and lease expiry under partition is genuinely hard.
- **Clock drift, jumps & corrections.** Components must declare whether they tolerate a wall-clock jump; the OS should never silently hand jumpy time to code that assumed monotonicity.
- **Zero value.** A zero clock capability is the inert null clock (shape 2): holding no clock is exactly the deterministic, time-independent default, and a zero `Duration` is a valid 0 (shape 1), so a zeroed timer fires immediately rather than erroring ([[omega_substrate]]).

## Key Questions

- What is the minimal default — does an ordinary component get *any* clock without asking?
- **`Clock::Trusted` attestation — decided** (see mechanism): anti-rollback monotonic counter (secure element/TPM/TrustZone) + Roughtime-style signed network time + measured-boot-attested keeper; RTC is only the cold-start baseline.
- **Virtual-time composition — resolved (subsumed).** A component's time-frame *is* its clock capability, and it is Matrix-scoped. Within a frame, time is consistent; **crossing out of the Matrix is the distributed case** (no global now — already handled, [[distributed_boundary]]); a *local* cross-frame mismatch arises only from a **deliberately-granted cross-frame capability** (a virtual-clock component handed a *real* service, or an un-owned differently-dilated child) — never a surprise, since the granter chose it. The deterministic-test rule follows: hand the component a virtual clock *and* virtual versions of its dependencies, and it stays in one coherent frame; hand it a real cap and that call is a real-world side effect by your own setup. The one genuinely-hard residual — cross-*machine* causality with no global now — lives in [[distributed_boundary]].
- What is the correct lease-expiry semantics when the only available clock is unreliable or partitioned?

## Omega Leverage

- Clock access is a **capability** plus reach to the **`Clock` boundary-service identity** — both already in Omega's vocabulary, so time appears in the authority graph and reach ceiling with no new machinery ([../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- A component with no clock capability is **statically** provable to be time-independent — the foundation of deterministic simulation.
- **Virtual time** is just a different `BoundaryProvider` behind `Clock::Read`; the test harness swaps the provider, the component never knows ([[testing_and_simulation]]).
- Omega does **not** model the *trust level* of a clock or distributed causality semantics — Cathedral adds the clock taxonomy and the trusted-time attestation as runtime/provider structure.

## Open Questions

- "Trusted time" on commodity hardware — **resolved (above):** best-effort but *attestable* — anti-rollback monotonic counter (defeats the load-bearing rollback attack) + Roughtime signed/cross-checked accuracy + measured-boot keeper; never perfect, but rollback-resistant and provable, which is what expiry decisions actually need.
- Should monotonic time be the universal default and wall-clock the privileged exception, inverting the legacy convention?
- How much of distributed time belongs here vs. in [[distributed_boundary]]?

## Related
- [[capability_model]] — clocks are capabilities in the authority graph.
- [[secrets_and_keys]] — trusted time underpins cert/replay decisions.
- [[distributed_boundary]] — causality and lease expiry without a global now.
- [[testing_and_simulation]] — virtual time as the basis of deterministic tests.
