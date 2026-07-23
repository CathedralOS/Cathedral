# Cathedral
A blazing fast & safe operating system built over a co-developed language called [Omega](../Omega/README.md).

Entire classes of frustrations, problems, and disasters are structurally impossible. Authority is always an explicit, held, revocable capability, leading to many benefits:
- **Instant file searching.** The underlying storage for files is a database, which means database speeds for file operations. Searching and filtering files is near-instant.
- **Instant file copying.** Copying a file, no matter how large, is instantaneous. The folder hierarchy is decoupled from the underlying file storage, so you only pay a cost when the file is changed (copy-on-write), or when the file is copied to another storage device.
- **Automated file history, replication.** Files have automated history checkpoints, again thanks to our copy-on-write database backing. We get this for free. Files can be set to replicate across drives, meaning your documents can exist in all of your hard drives simultaneously for reliable local backups.
- **Native sandboxing without container machinery.** A nested Cathedral instance, referred to as a Matrix, replaces Docker-style isolation for native software without requiring a guest OS. These cannot leak into the owning operating system. Even the main desktop is a Matrix. Detecting whether we are "in a Matrix" is almost impossible for malware, because everything is. We control what goes into and leaves Matrices. This works because the filesystem can share all system files as read-only. Virtual machines remain a compatibility tool for foreign operating systems.
- **No untrusted file access.** If a program wants access to a file or folder, it must be explicitly given and granted. A program can never take more than it was given.
- **No clutter.** There are no system directories littering your hard drive. System files are a completely separate "realm". From the user's point of view, the disk is fully theirs to use. Apps do not write to global directories, they own a realm nested under their binary.
- **A crash never corrupts your data.** Storage is one transactional, versioned object store: power loss leaves you at your last consistent commit, and any object rolls back to the state before a bad change — git-style history, built into the filesystem.
- **Update almost anything without rebooting.** Drivers, services, and compatible privileged-core changes hot-swap live, with rollback. A change that cannot prove a safe live path defers to reboot.
- **Ransomware is contained and recoverable.** No program can touch a file, device, or network it wasn't explicitly handed. A malicious download, a backdoored dependency, or a hijacked AI agent can damage only granted writable data; each grant is attributable and revocable, while versioned storage preserves recovery points.
- **AI agents you can hand real power.** An agent holds the *right to use* a secret, never the secret's bytes. A prompt-injected agent *cannot* exfiltrate keys, because the bytes never enter its reach. The worst a compromised agent can do is limited to "use this key, read-only, rate-limited, within this sandbox."
- **A buggy driver kills its device, not your machine.** Drivers run confined in user space behind a mandatory IOMMU.

Underneath, every OS contract is rebuilt on capabilities, proof-carrying components, typed state, explicit migration. In other words, the system is coherent by construction rather than by convention.

## Cathedral's bet

The wager, stated plainly:

> Every traditional OS domain has a legacy contract. Most of those contracts cannot answer simple questions: *who can do what, why, through which path, and can I revoke it safely?* Cathedral should answer them by construction, because authority, effects, protocols, and state evolution are visible to the compiler before a single byte is emitted.

Cathedral reconsiders *every* legacy OS contract, not one. Storage becomes a single content-addressed database of typed objects, partitioned into capability-rooted realms with no global root; authority is capabilities all the way down, with nothing ambient; IPC becomes typed protocol invocation; drivers run as confined, restartable user-mode components behind a mandatory IOMMU; identity becomes confined "Matrix" worlds; and persistence, install, upgrade, and even the running privileged core become declarative, hot-swappable state transitions.

One technique among these — a single address space with language-level isolation and live component replacement, used for the *proved* core — shares ground with [Theseus](https://www.theseus-os.com/). It is one idea here, not the thesis. The thesis is the wager above: that authority, effects, protocols, and state evolution are visible to the compiler before a single byte is emitted, so the OS can answer *who-can-do-what-why-and-can-I-revoke-it* by construction.

## Status

Cathedral is primarily in the **design and early boot** phase. The Omega-emitted
UEFI path now boots under QEMU/OVMF, exits firmware, mints a provisional first
physical extent, writes through a 16550 UART, and idles on `hlt`. There is not
yet a running kernel, scheduler, component runtime, or production driver.
Current work continues to specify each meaningful contract before the wider
system hardens around it.

If a design chapter and an implementation disagree, the chapter is the bug
report, not the law. Outside the narrow boot path, the chapters remain the
system's primary specification.

## Repository layout

The source tree is being populated milestone by milestone. The root separates the OS from everything *about* it:

- **`wiki/`** — design truth (why it's built this way).
- **`source/`** — the OS itself, split by trust: `contracts/` (the frozen ABI), `core/` (the proved kernel — the TCB), `foundation/` (the kernel-safe shared library), then userspace `services/`, `drivers/`, `libraries/`, `applications/`, and the `boot/` firmware seam.
- **`tools/`** — host-side tooling that never ships.

Directories under `source/` are created only when real code or contracts land.
The full plan, the dependency law, and the placement rules live in
[`wiki/architecture/repository_layout.md`](wiki/architecture/repository_layout.md);
the trusted set is enumerated in [`wiki/architecture/tcb.md`](wiki/architecture/tcb.md);
the decision is locked in [ADR 0001](wiki/decisions/0001-repository-layout.md).

## The design wiki

All design lives under [`wiki/design/`](wiki/design/design.md). Start at the index:

- **[Cathedral Design Index](wiki/design/design.md)** — the reading path, the chapter map, and the template every chapter follows.

The chapters are organized into parts:

- **Part 0 — Foundations**: the thesis, the Omega substrate, and shared vocabulary.
- **Part 1 — Authority & Trust**: the capability spine, identity, secrets, privacy.
- **Part 2 — Components & Execution**: the component model, scheduling, memory, time, failure, power.
- **Part 3 — Communication**: typed IPC, networking, the distributed boundary.
- **Part 4 — Storage & State**: the filesystem-as-database, transactions, configuration, live migration.
- **Part 5 — Lifecycle & Privileged Core**: packages, updates, drivers, boot, kernel.
- **Part 6 — Human Surface**: windowing, permission UX, media, naming, multi-tenant, web.
- **Part 7 — Observability & Governance**: introspection, compliance, telemetry, the store, extension boundaries.
- **Part 8 — Developer & Verification**: tooling, the debugger rethink, simulation, legacy compatibility.

## Relationship to Omega

Cathedral does not get to assume a finished language. Where a chapter needs a language feature that Omega does not yet have, it says so and links the relevant Omega language-guide chapter. The two repositories evolve together: Omega provides the proof, effect, capability, and versioned-data machinery; Cathedral is the first system large enough to put real pressure on it.

Key Omega chapters Cathedral leans on heavily:

- [Capabilities, Effects, And Boundaries](../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)
- [Domains](../Omega/wiki/language_guide/chapter_8_domains.md)
- [Machines](../Omega/wiki/language_guide/chapter_3_machines.md) and [States And Transitions](../Omega/wiki/language_guide/chapter_4_states_transitions.md)
- [Versioned Data And Machine Replacement](../Omega/wiki/language_guide/chapter_22_versioned_data.md)
- [Wire Protocols](../Omega/wiki/language_guide/chapter_21_wire_protocols.md)
- [Proof Obligations](../Omega/wiki/language_guide/chapter_9_proof_obligations.md)
