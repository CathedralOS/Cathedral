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
      contracts/           Public kernel ABI (implemented in Rust lab)
      boot/                Firmware entry and handoff
      core/                Scheduling, memory, capabilities, IPC, admission
      arch/                CPU mechanisms and privileged board adapters
    platform/
      contracts/           Public service protocols (implemented in Rust lab)
      services/            Shared system services (planned)
      drivers/             Device programs and pure hardware facts
      libraries/           Reusable algorithms, protocol clients and ports
    contracts/             Legacy Omega external UEFI declarations
    foundation/            Kernel-safe shared primitives (planned)
    distribution/          Cathedral's one built-in distribution
      profile.json         Boot composition consumed by the host harness
      shell/               Desktop experience (planned)
      settings/            User-facing configuration (planned)
      applications/        Bundled apps; Colors, Notes and chooser run in Rust OS
      libraries/           Distribution UI components, native backend and fonts
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
mechanisms and privileged board adapters. In the Rust lab, `arch/board` owns
concrete controllers and fixed board addresses; `arch/isa` owns CPU-specific
translation and context implementations; `core/user_tasks/address_space`
owns process memory layout, frame lifetime and checked copies. CPU backends
encode translations and saved contexts. A directory move does not create a separate kernel binary: the Rust
UEFI executable still composes entry and post-firmware startup through a small
orchestrating `main.rs`.

The kernel can boot independently of the shared platform. In the Rust lab,
ordinary boot optionally admits a host-supplied initial program and bounded
launch authority. `distribution/desktop` chooses when to start and use the selected
platform providers. The profile chooses executable artifacts; the kernel enforces
their grants. Exhaustive bring-up composition lives under `kernel/boot/uefi/lab`
and is compiled only in smoke builds. The bounded launcher supports up to seven
approved child slots, whose launch grants can be lent recursively at child
admission; this is not yet a general service graph or admission contract. The built-in distribution launches display, input, storage
and applications; separate lab profiles exercise other approved graphs.

`platform/` is the shared OS base outside the kernel: display/capture, normalized
input, audio, clipboard, accessibility, storage, networking and service lifecycle.
Most of these services are not implemented yet. Stable interfaces, maintained
implementations and conformance tests belong here. A desktop profile can require
a common set of interfaces independently of its shell.

`distribution/` owns presentation, interaction design, defaults and bundled apps.
Its `libraries/omega-ui/` contains the checked-in Omega component source and
native backend; the Rust mirror owns the reusable app-facing adapter. UI builds
use this source directly, rather than a sibling repository or platform-to-distribution
dependency.
UI development, font generation and capture tools live in `tools/omega-ui/`;
their host artifacts belong in the ignored `build/omega-ui/` tree.
The installed Colors program now lives in `source/distribution/applications/colors`.
Its Rust executable only supplies startup and the common
`platform/libraries/omega-app` host. That host performs requested scoped IPC and
buffer custody; app state, input decisions, storage schema and presentation stay
in distribution-owned Omega. This temporary native bridge is not the final
Omega application ABI. The Rust Colors library remains a mechanism-lab probe.
Notes and the chooser use the same host. Their sessions, settings, input and
painting live in distribution-owned Omega. The chooser's separate pure policy
export remains only for decision tests and explicit Rust fault probes; ordinary
F5 calls that policy directly from Omega. See the
[chooser boundary](../../source/distribution/applications/chooser/README.md).
The Rust stock matrix is also a distribution program: `distribution/matrix/`
owns its launcher/app roles, saved desktop configuration, child control
router and recovery choices. Its saved configuration validation and placement
now execute the shared [Omega configuration](../../source/distribution/matrix/configuration/README.md);
the chooser calls the same codec directly. Stock close/loss aggregation and
wait/refuse/stop decisions also execute [Omega lifecycle policy](../../source/distribution/matrix/lifecycle/README.md).
Bounded launcher retry and fallback decisions execute
[Omega recovery policy](../../source/distribution/matrix/recovery/README.md).
Local picker/background retry budgets execute
[Omega startup policy](../../source/distribution/matrix/startup/README.md).
Authenticated IPC custody, stale reply draining and task reclamation stay in Rust.
The Rust data representation remains
for installer/tools and independent conformance checks. Its extension contract lives beside it in
`distribution/matrix/contracts/`. Other distributions need not implement it.
`platform/libraries/matrix/` supplies reusable local authority, input, geometry,
surface, storage-scope and task-management state with caller-selected child
capacity. It imposes no launcher, desktop strip or persisted configuration.
The stock program's IPC loop still composes these mechanisms directly; a
general-purpose matrix server runtime is not yet a separate platform package.
The independent Rust `distribution/lab/matrix-composition/matrix` program uses
the shared runtime server and matrix commands for three local peers without any
stock distribution import. The same proof now also runs an
[Omega matrix program](../../source/distribution/lab/matrix-composition/README.md)
over a private lab bridge to those mechanisms. It chooses admission, service
grants, placement and lifecycle; the bridge retains authenticated request custody.
The common app host has not acquired matrix-specific operations.
This verifies that subset over actual IPC; it does
not yet share the stock program's rendering or asynchronous upstream adapters.
The Omega proof also accepts a real parent-supplied broker and attenuates its
view/input to three peers; its private synchronous adapter is a boundary proof,
not the stock desktop event loop. Child connection
construction, spawn and failure rollback are shared through
`platform/libraries/user-runtime/matrix/admission/`; each program retains code,
budget, placement and recovery policy. Scoped service acquisition also lives in
platform contracts/runtime; the stock owner resolves declarations and selects the
per-child offer. The kernel transfers endpoints without knowing service names.

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
distribution       -> platform packages, kernel/contracts, foundation
platform           -> platform packages, kernel/contracts, foundation
kernel/core        -> kernel/arch, kernel/contracts, foundation, pure hardware facts
kernel/arch        -> kernel/contracts, foundation, pure hardware facts
kernel/boot        -> kernel packages, foundation
kernel/contracts   -> foundation only; no implementation packages
platform/contracts -> kernel/contracts, foundation; no implementation packages
foundation         -> no implementation packages
```

- Kernel and platform never depend on distribution implementations.
- Platform and distribution may import the independently built `kernel/contracts`
  package, never kernel implementation packages. Public definitions do not link
  the kernel into an app. Operations and capabilities are explicitly supplied.
- The kernel does not import platform service protocols. Shared display geometry
  belongs to the kernel framebuffer boundary; drawing requests belong to the
  platform display protocol.
- A contract package cannot import its provider implementation. Nesting it under
  its owner identifies responsibility, not a dependency on that owner's executable.
- Distribution extension contracts may depend on public platform/kernel contracts,
  but cannot depend on their implementations. Platform and kernel cannot import
  distribution extensions. The layout checker enforces the stock matrix boundary.
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

Cross-component contracts live with the subsystem that defines their semantics.
In Rust, `kernel/contracts` owns the syscall ABI, page capabilities and framebuffer
aperture record. `platform/contracts` owns display/compositor, input, matrix and
catalog protocols, independently of their implementations. Distribution-specific
coordination stays under distribution. A contract can be experimental: naming it
as a boundary does not freeze its ABI or promise stability.

Internal records and ordinary library traits need not become public OS protocols.
The boot memory inventory belongs to `kernel/core/memory/inventory.rs`; firmware
adapters construct it and the core validates it. The synchronous block `Device`
trait belongs to `platform/libraries/block-device`, shared by drivers and storage
adapters. Neither requires a universal contracts package.

Rust's old top-level contracts crate has been retired. Omega's existing
`source/contracts/uefi` holds external firmware declarations and remains at its
current paths to preserve port provenance; it is not a precedent for collecting
new kernel/platform protocols in a universal root. Its existing dependencies remain
checked separately. Private protocols, manifests, proofs, migrations and tests
stay with their owner. `foundation/` is reserved for
small kernel-safe primitives without ambient authority. Entry files follow the [discoverability architecture](discoverability.md):
show real orchestration at one level, with named routes into the next owners.

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
