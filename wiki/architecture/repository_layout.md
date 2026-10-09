# Repository layout

**Adopted 2026-10-09.** Cathedral is one monorepo containing its kernel, shared
platform, and one built-in distribution. [ADR 0002](../decisions/0002-kernel-platform-distribution.md)
supersedes ADR 0001's flat grouping. `source/` contains Omega target code;
`source-rs/` mirrors the same boundaries for the executable Rust lab.

## The tree

```text
Cathedral/
  source/                  Omega implementation: everything that ships
    kernel/
      boot/                Firmware entry and handoff
      core/                Scheduling, memory, capabilities, IPC, admission
      arch/                CPU mechanisms, when a separate backend exists
    platform/
      services/            Shared system services (planned)
      drivers/             Device programs and pure hardware facts
      libraries/           Reusable algorithms, protocol clients and ports
    contracts/             Shared ABI and platform interface vocabulary
    foundation/            Kernel-safe shared primitives (planned)
    distribution/          Cathedral's one built-in distribution
      profile.json         Boot composition consumed by the host harness
      shell/               Desktop experience (planned)
      settings/            User-facing configuration (planned)
      applications/        Bundled applications (planned)
  source-rs/               Rust lab with the same ownership boundaries
  tools/                   Host builds, image assembly, checks and simulators
  wiki/                    Contracts, architecture, rationale and decisions
```

Directories appear with real residents. Distribution exists today because its
profile is consumed by the boot harness. No placeholder desktop packages are
created. Omega currently keeps CPU policies in `kernel/core` and hardware facts
in `platform/drivers/facts`; Rust already has a separate `kernel/arch` crate.

There is one `distribution/`, singular, with no multi-distro registry or plugin
framework. A fork replaces it and keeps the kernel/platform contracts. Defaults,
application selection and workflows belong there. Host tools assemble its
selections into images; the selections themselves live with target source.

## Responsibilities

`kernel/` owns privileged execution, including the firmware seam. `kernel/core`
stays firmware-neutral; `kernel/boot` adapts firmware, and `kernel/arch` owns CPU
mechanisms. A directory move does not create a separate kernel binary: the Rust
UEFI executable still composes entry and post-firmware startup through a small
orchestrating `main.rs`.

The kernel can boot independently of the shared platform. In the Rust lab,
ordinary boot optionally admits a host-supplied initial program and bounded
launch authority. `distribution/init` chooses when to start and use the selected
platform providers. The profile chooses executable artifacts; the kernel enforces
their grants. Exhaustive bring-up composition lives under `kernel/boot/uefi/lab`
and is compiled only in smoke builds. The bounded launcher supports up to three
independent approved children; this is not yet a general service graph or
admission contract. The built-in distribution launches display and input.

`platform/` is the shared OS base outside the kernel: display/capture, normalized
input, audio, clipboard, accessibility, storage, networking and service lifecycle.
Most of these services are not implemented yet. Stable interfaces, maintained
implementations and conformance tests belong here. A desktop profile can require
a common set of interfaces independently of its shell.

`distribution/` owns presentation, interaction design, defaults and bundled apps.
Capture pickers, settings screens and desktop controls use platform contracts.
Authority semantics, protected surfaces, observation visibility and the operator's
recovery/revoke path remain enforced by trusted platform owners. Styling a prompt
does not grant permission to bypass its authority checks.

The [compositor chapter](../design/part_6_human_surface/00_windowing_and_compositor.md)
has an older permanent-stock-chrome design. Its trusted recovery requirement
remains relevant; everyday stock desktop source now belongs to `distribution/`.
This migration does not implement or finalize the future compositor protocol.

## The dependency law

```text
distribution       -> platform public packages, contracts, foundation
platform           -> platform packages, contracts, foundation
kernel/core        -> kernel/arch, contracts, foundation, pure hardware facts
kernel/arch        -> contracts, foundation, pure hardware facts
kernel/boot        -> kernel packages, contracts, foundation
contracts          -> no implementation packages
foundation         -> no implementation packages
```

- Kernel and platform never depend on distribution implementations.
- Platform and distribution never import kernel implementation packages. They
  target contracts and receive explicitly supplied operations or capabilities.
- Profiles select the kernel entry as composition data, not a userspace import.
- The Rust profile also selects a separately compiled distribution program.
  Host tooling supplies its ELF artifact to boot; the kernel's loader consumes
  bytes through the experimental user boundary, without importing distro code.
- Pure hardware facts are an explicit kernel dependency on
  `platform/drivers/facts`; they hold no authority and import no kernel code.
- Rust boot currently links the platform UART for diagnostics. This exact
  bootstrap exception is checked by the layout tool. The UART has no kernel
  dependency: boot supplies its port read/write operations. It still executes
  privileged until driver isolation exists.
- Omega's existing boot-to-core extent-provider edge is an explicit composition
  dependency, not permission for userspace imports of core.

Run `python tools/source-layout/check.py` to check source roots, boot profiles,
Omega package paths and Rust workspace edges. This does not prove runtime
isolation or capability custody.

## Trust, ownership and provenance

The folder identifies responsibility, not privilege. Some platform services mint
authority and belong to the [enumerated TCB](tcb.md). Moving them outside the
kernel does not remove them from the audit surface. The Rust lab now runs probes
and a separately compiled distribution program in private ring-3 address spaces.
Its boot/core and bootstrap UART remain privileged; the folder boundary alone
still supplies no containment or capability authority.

Frozen cross-component ABI belongs in `contracts/`. Private protocols, manifests,
proofs, migrations and tests stay with their owner. `foundation/` is reserved for
small kernel-safe primitives without ambient authority. Keep entry files thin.

Ports stay with their eventual owner, with `PORT.md`, mappings, tests and notices.
Studied checkouts stay ignored under `reference_code/`; untypeable staged ports
stay outside production build roots. See the [porting policy](prior_art_and_hardware_facts.md).

Translated Omega bodies remain byte-identical through this move. Build manifests
and tools change to resolve new paths. Historical verification receipts describe
the exact inputs tested at the time; relocation is not a new semantic run, and
recorded digests must not simply be replaced with today's hashes. Original path
comments in unchanged Omega files remain provenance.

Unit checks co-locate with their owner; host canaries and boot tools live under
`tools/`. The Rust smoke boot exercises memory reclamation, task preemption,
user-fault containment and restricted ELF loading.
The Omega harness selects its own entry from `source/distribution/profile.json`.
