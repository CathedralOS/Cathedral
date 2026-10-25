# Installation and component-owned activation

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | None |
| Last reviewed | 2026-10-25 |

## Scope

The owner ratified independent installation and component-owned activation for
applications, distribution services, platform services and kernel artifacts.
Installation does not promise that every running instance adopts the new version.
No universal hot-swap engine or global live-adoption progress tracker is required.
This contract specifies ownership and observable outcomes, not a package format,
syscall ABI, migration algorithm or implemented updater.

## What the updater completes

An authorized updater stages and validates candidate artifacts under installation
policy, installs new immutable versions, and publishes the appropriate future
launch or boot references. It does not overwrite executable backing beneath a
running instance. Old and new versions may coexist; existing execution, mappings
and other live obligations retain the old backing until safe reclamation.

Reference publication selects an admitted complete artifact, never a partially
downloaded image. Failure before publication leaves the prior selection usable.
Installing a candidate and selecting it are distinguishable outcomes; a candidate
may be retained without becoming the selected launch version. This contract does
not require a transaction across every package in an OS update.

The updater can report artifacts installed, references changed, and authorized
update notifications submitted or failed. Successful notification submission is
not receipt, preparation, migration, health or successful live activation.
Notification failure must not be hidden or relabel an installed artifact as absent.
The updater's completion does not wait for every live instance to acknowledge it.

Installation remains subject to integrity, trust, authority and compatibility
admission. This separation neither bypasses admission nor authorizes installer
code to run arbitrary application migrations with ambient privilege.

## Control requests and authority

A user gesture through OS chrome may authorize an update, but authority is still
explicit. Installing artifacts, changing launch references, managing instances,
signalling a control connection and observing lifecycle are distinct permissions.
An installer does not gain termination or migration authority merely by writing
a new executable version.

Running-instance owners, normally their matrices or supervisors, receive version
availability and apply their own activation policy. An application opts into an
ordinary update-control connection with explicit notification rights. The owner
retains the sending role; the application services the receiving role. The
[notification facility](notifications.md) supplies remembered signals and parking;
there is no update-specific kernel signal, implicit subscription or injected
handler in interrupted code. Shared protocol data identifies the request and
replacement; a coalescing wake alone cannot encode every requested update.

An application need not already be parked when notified. It handles the request
when it services its connection. Unresponsive or nonparticipating applications do
not make completed installation fail. Retry and deferred-activation policy belong
to the owner; lack of acknowledgement is not readiness.

## Activation belongs to the component owner

The owner may leave an existing instance alone until natural exit, ask it to save
and restart, or use a live module-replacement protocol that the component actually
supports. It may prompt the user. Forced termination requires separate authority
and cannot be presented as successful state preparation.

For a whole executable, a cooperative lifecycle may be:

```text
owner -> old instance: prepare to stop for update
old instance -> owner: ready with checkpoint, or not ready
owner: stop old instance; observe actual exit
owner: launch selected new executable with authorized saved state
new instance -> owner: ready, if this service contract requires readiness
```

These are protocol roles, not prescribed message spellings. Any acknowledgements
used by the owner are bound to the exact connection, process incarnation and
request; old replies cannot complete another replacement. App readiness remains
an attributable app claim, not something the kernel infers from saved bytes.
No global updater acknowledgement or per-instance convergence metric is required.

For a module replacement, the component's loader/coordinator handles entry control,
outstanding calls, state migration, dispatch cutover and retirement. Retaining an
arena's bytes alone does not establish handle validity, callback lifetime or
successful migration. The kernel can observe its own admission/execution events;
it does not infer that module state and application traffic were migrated correctly.

The [matrix lifetime rule](components_and_matrices.md#matrix-failure-and-subtree-lifetime)
still applies: stopping or restarting a matrix tears down its owned subtree.
Installing a new matrix executable does not stop the current one, adopt its live
children into a replacement, or create a new exception to that rule. Independent
instances update independently only where their actual ownership boundaries allow.

## Compatibility and deferral

Compatibility-preserving independent activation is the normal requirement for
apps and distro/platform services. Versioned wire protocols must permit the
supported mixed-version population; sharing a message encoding alone does not
establish that compatibility. Persistent-state formats and independently held
resources must also remain usable by the versions still allowed to access them.

A coupled incompatible change needs an explicit bridge, compatible coexistence,
or a coordinated restart/migration scoped to the affected set, possibly at reboot.
It is not permission to silently break old running clients. Such a workflow is
separate from ordinary installation, not a universal global-quiescence phase.

The same installation boundary covers kernel modules and kernel images. Live
replacement is optional. If a supported safe transition cannot be completed,
retain the old execution and defer activation; a kernel artifact may take effect
on a later boot. Installing it neither proves live-kernel replaceability nor
guarantees that reboot solves an incompatible durable-state migration.

## Enforcement and orchestration

Orchestration belongs to ordinary platform services and component lifecycle
owners. Kernel code owns the privileged mechanisms needed to enforce system
invariants, not application-specific migration or update policy.

| Work | Orchestration owner | Enforcement boundary |
| --- | --- | --- |
| Install / load | Updater and loader services prepare and select candidates | Admission, authorized executable mappings and task creation |
| Replace a live component | Its lifecycle owner coordinates drain, state and traffic | Execution control, mapping/entry lifetime and authorized grant changes |
| Storage transaction / recovery | Storage service coordinates records and durable publication | Its admitted storage rights and any explicitly required kernel transition |
| Attestation | Authorized service assembles the report | Authentic measurements, protected keys and authoritative kernel observations |

Moving orchestration out of the kernel does not automatically remove it from the
TCB. Trust follows the authority held and claims enforced; admission cannot accept
an untrusted service's assertion as evidence of enforcement. Report kernel trust
and privileged platform trust separately. Exact provider/ABI shapes remain
engineering work within this boundary.

Every actual replacement must preserve old code/backing while it remains usable,
the contracts of its consumers, and a defined recovery or shutdown path. Traffic
redirection, standby buffers and preemption are mechanisms, not proof of quiescence.
If preparation fails before cutover, the old instance remains usable. Once a
declared irreversible step commits, the component's recovery contract governs;
the updater does not promise universal rollback.

This does not introduce a transaction atomically spanning durable storage and
kernel authority. A workflow requiring one must establish its commit, custody and
recovery semantics rather than treating separately successful calls as atomic.

## Implementation acceptance

The existing loader, immutable-image admission, notification and lifecycle work
provides prerequisites, not evidence that this complete update workflow runs.
[The implementation outcome](../../../TASKS_RUST.md#independent-installation-and-component-owned-activation)
must demonstrate a new launch using v2 while v1 remains valid, authorized control
notification without mandatory acknowledgement, safe old-version reclamation,
and independent instance deferral. Optional module/kernel live replacement is not
an acceptance prerequisite for that ordinary update path.
