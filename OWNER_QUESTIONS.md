# Owner questions

Only unresolved owner-level Cathedral semantic, compatibility, architecture,
or trust decisions belong here. Settled decisions live in the
[specification](wiki/spec/README.md); implementation work and deliberately
deferred research do not.

Before adding a question:

1. Name the concrete product or implementation requirement that is blocked.
2. Cite the specification owners and machine-readable contracts checked and
   found silent.
3. Separate the owner choice from ordinary engineering or extraction work.
4. State the viable choices, their invariant and compatibility consequences,
   and a recommended answer.

A missing specification page is not by itself an owner question. Neither is an
implementation difficulty. Record design conflicts and oddities as they are
encountered when they expose an owner choice affecting an active requirement
or architecture boundary. Ordinary documentation corrections can be fixed
directly; recommendations here remain proposals until accepted.

Questions are a mutable decision queue, not stable contract identities. Code,
tests, and settled documentation must cite the specification clause produced by
the answer rather than an owner-question number. Resolving a question updates
the specification and affected source contracts in the same change, then
removes the question. Git retains the discussion history.

Existing `Key Questions` and `Open Questions` in design chapters are design
inventory. They are not automatically owner questions under this narrower
standard.

## Open questions

### The stock matrix outgrows one Omega component

**Blocks the stock matrix migration.** The standalone Omega stock matrix is
one native program, packaged as one independent Omega component. The Omega
compiler refuses a component whose description, which embeds the compiled
artifact of the whole program closure, exceeds 8 MiB
(`MAX_COMPONENT_DESCRIPTION_BYTES` in Omega's component description). The
matrix as of the funding step is 8,193,855 bytes, 194 KB under the bound. The
next slice, the chrome's app entries and selection, takes it over; that work
waits on the `stock-matrix-workspace` branch. Size tracks the number of
machines in the closure, about 4 KB each over some 2,000 machines across the
process, the stock policy, the World and the runtime. The Rust host's
remaining responsibilities would add roughly 1.5 MB more, so trimming code
cannot make room. The [process README](source/distribution/matrix/process/README.md)
and the Omega toolchain notes in the [handoff](TASKS_RUST.md) are silent on a
program-size ceiling.

Choices:

- Raise the bound in the Omega producer Cathedral pins, for example to 32 MiB,
  and requalify that producer. Nothing in Cathedral's architecture changes;
  the cost is a larger decode allowance wherever a description is verified.
- Make descriptions compact in Omega, for instance by interning the repeated
  nominal type identities, which are about a third of the printable bytes.
  Same effect as raising the bound, and smaller artifacts, but it is
  language work with its own review.
- Split the stock matrix into several processes, for instance storage relay,
  presentation and child supervision. This changes the architecture purely to
  fit a tool limit: one trust unit becomes several, and every input, paint and
  storage hop gains an IPC crossing.
- Let the packager link more than one independent component into one process
  image. Tooling work of unknown size, since installation binds each
  component's imports.

Recommended: raise the bound now so the migration continues, and treat compact
descriptions as Omega's own follow-up. The matrix stays one process.

### Exclusive audio and the reserved system channel

**Deferred; does not block shared playback.** The owner ratified the
[shared-buffer playback baseline](wiki/spec/human_surface/audio.md): caller-funded
sample rings, authority-preserving forwarding and final platform mixing. There is
no promised exclusive path or automatic raw device handoff.

A later exclusive-output proposal must distinguish a service admitting only one
stream from delegating actual device control. On an output without an independent
enforced path, unrestricted handoff cannot also guarantee reserved mixing.
That proposal must identify its customer, measured need, delegated authority,
reserved-channel policy and failure/reclamation behavior.

No exclusive-mode option is selected. Keep it out of the shared playback
acceptance; return for a separate decision when a concrete use case requires it.

### Validation scope for an optional PCID optimization

**Resolved boundary.** The owner accepted [hardware-separated programs](wiki/spec/execution/isolation.md)
on 2026-10-23 and removed PCC-based single-address-space isolation from the
current roadmap. This replaces the former default-isolation question. PCID does
not require merging program address spaces; its test coverage is a separate choice.

**Requirement and choice (deferred).** The lab evidence below reports that the
current emulator path cannot exercise the proposed PCID optimization, while WHPX
can. Before enabling such a path, decide whether hardware/hypervisor-only coverage
is acceptable with a retained non-PCID fallback, or require broader target coverage
first. This does not block current hardware-separated execution or coordination.

**Recommendation (unaccepted).** Leave the optimization disabled until there is a
concrete implementation and validation plan. Do not infer approval of a different
kernel page-table layout or weaker isolation from the accepted program boundary.

### Shared render-cache sponsorship and stronger residency guarantees

**Remaining choice; not a blocker for the baseline.** The owner accepted both
drawing commands and completed images, destination-host-funded or explicitly
sponsored uploads, and omission of unavailable draws with submission-specific
feedback. Active acquisitions retain backing until readers finish. These rules
live in the [resource contract](wiki/spec/human_surface/rendering_resources.md).

A production shared image/font cache may still need a policy choosing the paying
account when several consumers retain one derived allocation, and any availability
promise stronger than registration plus bounded acquisition. The existing Rust
page contract does not implement that shared-cache service or GPU residency.

Do not reopen missing-resource appearance or remote upload ownership to implement
the accepted route. Bring a concrete shared-cache or residency-guarantee customer,
funding policy and reclamation acceptance before selecting those additional
guarantees. GPU import, synchronization and presentation efficiency remain open
engineering, not evidence that caller-funded local rendering must wait.
