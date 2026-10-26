# Components and matrices

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-26 |

## Scope

This contract establishes matrix policy ownership and supplied authority,
following the owner's 2026-10-18 clarification. It does not specify a universal
matrix configuration format, the complete component lifecycle, or realm
encryption and locking protocols.

The [installation and activation contract](updates.md) separates installing new
executables from replacing live instances. Matrices own their children's update
policy within supplied authority; installation grants no new lifecycle power.

## A matrix is an ordinary program

A matrix is a program acting as a host and attenuator for children. Its parent
determines the authority available to it; the matrix determines what it supplies
to its children within that authority. Hosting a matrix uses the same authority
boundary as hosting another program.

The [program isolation contract](isolation.md) requires hardware separation on
current targets; platform providers and matrices have no proof-based exemption.

The matrix may implement its policy in compiled code, configuration, or both.
Custom matrices need not implement the distribution desktop's settings schema.
They may deny a requested service, delegate an authorized restricted operation,
interpose their own provider, or synthesize a contained implementation. A
synthesized service gains no authority over the real resource merely by
implementing the same interface.

Delegation does not require every ancestor to forward every invocation. A
provider may enforce a delegated grant directly; an interposing matrix is an
implementation choice when its behavior requires mediation. Enforcement must
remain effective regardless of which path carries the request.

The [memory-budget contract](../resources/memory_budgets.md) makes this concrete:
backing allowances partition the supplied envelope, direct grants remain
kernel-enforced, and a matrix may withhold them in favor of an interposed
allocator. No compulsory per-allocation forwarding chain is introduced.

## Supplied interfaces do not reveal upstream construction

A child receives the observations and operations its supplied interfaces authorize,
not an implicit account of how the enclosing environment was constructed. Service
access does not grant inspection of upstream providers, host topology, hidden
participants or whether the environment is physical, virtual or synthesized.
Additional disclosure requires explicit authority; there is no ambient bypass for
anti-cheat, automation detection or sandbox detection.

This applies equally to an ordinary desktop, remote session, accessibility setup
or malware-analysis environment. Providers must honor their declared interface
semantics, but need not reveal the machinery behind them. The rule does not promise
perfect indistinguishability against timing or other authorized observations.
Owners may retain private enforcement information and present their own authorized
UI without thereby disclosing it to children. The
[input provenance rule](../human_surface/input.md#upstream-provenance-is-not-input-metadata)
is one application of this boundary.

## Matrix failure and subtree lifetime

The owner accepted on 2026-10-23 that a matrix is a lifetime boundary: when it
crashes or terminates, its owned children and all their descendants are torn down.
No descendant survives as an orphan for a replacement matrix to adopt. The
enclosing system must enforce this without relying on the failed matrix or on
cooperative child shutdown. This is a containment rule, not just a desktop policy.

Restart creates a new matrix and new child instances within newly admitted
authority. Durable state may be reopened through authorized storage; saved task
identifiers, handles or registrations do not resurrect the old subtree or confer
control over another live process. Persisted activation registrations describe
what to start, not surviving descendants to reclaim.

A service owned outside the failed matrix may continue running. Supplying its
interface to the matrix does not make that provider a descendant, and a child
cannot escape its lifetime boundary by opening IPC or registering with an
external service. Coordination's first participant is not a lifetime owner merely
because it accepts connections. There is no implicit promotion or adoption on
failure; this contract introduces no operation for moving a live descendant out
of its matrix.

Subtree death does not roll back completed external work or destroy backing still
owed to an authorized surviving reader. Resource reclamation must preserve those
obligations while preventing further execution of the dead subtree. Exact budget
refund routing and outstanding external-operation outcomes remain separate
accounting and service contracts.

## Policy ownership and persistence

The enclosing owner controls the matrix's granted capabilities and resource
ceiling. The matrix controls its internal arrangement within that ceiling:
launcher selection, startup children, placement, service bindings and allocation
of its available budgets.

When the matrix persists this arrangement, it is its own program state in
storage it is authorized to use. The matrix may delegate editing to a settings
program. Being a child confers no authority to edit the parent's configuration.
Access to configuration data and authority to change live grants are distinct.

Configuration requests authority; it does not create it. A request beyond the
available grants or budgets must be refused rather than silently widening
authority. Parent ceilings remain effective even when the matrix's code or
configuration requests otherwise.

Saving configuration changes the saved arrangement for subsequent starts.
Applying it to running children is a separate, explicit authorized operation,
which validates the requested transitions and handles affected children and
unsaved work. Saving alone does not silently terminate children or alter their
live grants. A UI may explicitly request both save and apply together; this
contract does not prescribe buttons or an application protocol.

## Supplied services and storage

Programs declare required and optional interfaces; the owner selects the actual
capabilities supplied at launch. An unsatisfied required import refuses launch;
an optional import may be absent. A declaration, persisted service name or
caller-supplied PID is not a grant. A scoped locator, if supplied, is itself a
service capability, not ambient access to every provider.

Executable identity, storage authority and authority to unlock sealed storage
are distinct. Running a matrix does not by itself authorize access to a sealed
realm. A supplied storage view cannot confer access outside its granted scope.
This contract does not require every matrix to receive the subtree beneath its
executable, or prescribe how a view is presented as `/`.

Secret-backed services supply authority to perform operations, not a means for
the receiving app or matrix to retrieve credential bytes. An app can receive a
restricted credential-using operation without holding the credential. This is
not a claim that every trusted implementation can operate without key material
in software, or that an authorized operation cannot be abused within its limits.
Unlocking a realm supplies authorized access to its contents; it does not imply
delivery of its encryption key to the matrix program.

### Storage access is demand-driven

On 2026-10-25 the owner rejected automatic parent-generated storage requests
through a child's binding to warm permission/containment checks. Granting a realm
permits access; it does not require an app or matrix to use that realm. Launch or
grant installation alone must not synthesize such a request as if the child sent it.

Run the necessary checks when an actual child storage request requires them.
Successful checks may be cached against the exact binding, scopes and validity
conditions; revocation, replacement and relevant changes invalidate them. Cached
containment is not permission to skip the operation's own authorization checks.
This does not prohibit admission validation or a parent's independently authorized
storage operations under its own binding. Future cache optimizations must preserve
request attribution and demonstrate a need, not presume every grant will be used.

## Multiple instances over one realm

The same program may be launched more than once with access to the same realm,
subject to the usual authority and resource admission. A second launch does not
implicitly clone the realm or become invalid merely because an instance exists.
This rule applies to matrices as ordinary programs as well as to leaf apps.

The program determines whether it supports concurrent use or rejects another
instance. Sharing a realm does not serialize the programs' operations or their
private state. This acceptance does not specify an exclusive-instance claim,
lock lifetime, discovery or activation protocol; those mechanisms remain to be
designed in detail. The accepted [coordination service](../communication/ipc.md)
supplies scoped introductions and app-controlled acceptance or refusal; its Rust
integration is outstanding. An independent copy remains an explicit storage operation.

## Scoped application bootstrap

On 2026-10-25 the owner selected on-demand syscall access to a launch-established
**Matrix connection**: the child's canonical connection to its supplied environment.
Broker names the service-resolution responsibility behind that interface, not a
required separate process. The owning matrix may implement it directly or arrange
an authorized provider. This is not the child's only permitted IPC connection.

A manifest declares required/optional imports; the launching owner resolves the
authority ceiling before launch. For a child supplied this interface, launch
provisions and initializes funded backing for the standard Matrix protocol and
installs the child's required mapping and notification rights before the child runs. The lookup
syscall exposes those existing resources, not
merely an endpoint requiring the child to allocate before it can communicate.
It is access-on-demand, not first-use allocation. An absent relationship reports
unavailable; requesting one cannot create authority or select another environment.

The launcher uses memory authority or an allocator connection it already possesses.
The first child allocator connection is likewise provisioned by the parent/provider
using existing authority; the child is not required to call that allocator before
it can reach it. Funding may come from the child's admitted connection budget, not
an arbitrary charge to unrelated accounts. This bottoms out at privileged startup's
initial resource authority. Zeroed connector bytes cannot create notification
objects or install authority. See [connection setup](../communication/ipc.md#matrix-access-and-connection-setup).

The Matrix protocol is scoped to this environment interface. It is not a mandatory
envelope for every acquired service: each service declares its own shared-storage
layout, direction, synchronization and lifetime requirements. Reusable transport
libraries may be selected without imposing rings, atomic headers or replies on
one-shot or one-way resource handoffs.

The kernel enforces access to this process's authorized bootstrap connection. It
does not interpret a rendering/storage service catalog or resolve platform import
records. The broker is ordinary userspace functionality, possibly within the
owning matrix. It authenticates the established connection, not a claimed PID or
manifest supplied in request bytes. Actual service acquisition then uses ordinary
IPC and returns connections preserving the owner's mediation and authority ceiling.

Acquisition identifies a declared import under that child's admitted contract,
not an ambient string lookup for arbitrary services. Friendly configuration names
do not grant authority or let another provider replace an existing selection by
registering the same name. Third-party providers require explicit authorized
selection and interface/version compatibility; those concrete registration formats
remain engineering work.

Repeated Matrix-connection access denotes the same logical connection, not a fresh
connection or extra service authority. The library retains and coordinates its use, including
request correlation. Handle representation and synchronization are engineering;
concurrent/repeated access must not allocate unbounded connections, reset protocol
state or revive a closed/revoked connection. Individual service connection creation,
sharing and transfer remain governed by that service's acquisition contract.

No broker parameter in authored main, reserved prefix in application argument bytes,
or special startup adapter is required. Do not retain a second broker-discovery
route through startup arguments as a speculative syscall optimization. Ordinary
program-specific arguments and explicit authority injection remain possible; this
decision does not define a universal application argument schema or change Omega's
general entry adaptation. Bootstrap and service protocol versions evolve independently.

A portable library may acquire its permitted service during subsystem initialization
and own the resulting connection, without requiring every caller to thread a broker
argument. Its platform binding performs acquisition; tests/embedding may explicitly
supply a provider. Neither convenience nor a matching interface grants authority.
Ordinary operations use the acquired connection without another broker lookup.
This does not promise automatic provider startup or perpetual availability.

The Rust [bootstrap contract](../../../source-rs/platform/contracts/services/README.md)
records the existing startup-endpoint implementation, which must migrate to this
on-demand access rule. Its current main wrapper is not the accepted end shape.
Direct startup grants remain necessary for privileged programs establishing brokers;
this does not mandate a registry in the kernel or eliminate explicit handles.

## Privileged startup

The privileged startup environment orchestrates resource owners and delegates
the first service connections. It need not be one matrix executable, and the
login picker is not the authority root. Drivers, storage, credential providers
and other services remain independently owned implementations.

A root Warden may hold hardware-backed authority outside user matrices. A
matrix may instead provide a Warden backed by authority it owns, or synthesize
a contained implementation. Receiving a Warden interface does not imply access
to the root Warden or its credentials.

## Claims not yet established

This contract does not settle object kinds/names, aliases, copy/reference
rewriting, a mandatory executable-to-realm relationship, CoW realm semantics,
or the trust and installation format of boot composition. It does not specify
Warden wire operations, key custody mechanisms, authentication, or the disposal
of plaintext held by running processes when a realm is locked. Those subjects
retain their existing design/specification owners.

## Implementation evidence

- The Rust [supervision model](../../../source-rs/kernel/core/user_tasks/lifecycle/supervision/supervision.rs)
  cancels owned children on supervisor exit or fault without running user
  destructors. It does not implement orphan adoption or production cooperative
  cancellation. The [recursive admission lab](../../../source-rs/distribution/lab/channels/README.md#recursive-child-admission)
  exercises subtree reclamation and returns lent budgets to the nearest surviving
  lender; that refund route is lab evidence, not an accepted general sponsorship rule.
- [Declared service imports](../../../source-rs/platform/contracts/imports/README.md)
  implement bounded required/optional binding and owner withholding. The record
  and endpoint bundle are experiments, not a frozen component manifest.
- [Matrix requests](../../../source-rs/platform/contracts/matrix.rs) and
  [child admission](../../../source-rs/distribution/matrix/service/children/children.rs)
  exercise owner ceilings, restricted launches and recursive hosting.
- The [distribution owner](../../../source-rs/distribution/desktop/session/shell/shell.rs)
  grants the user ceiling and requests startup. The
  [stock matrix](../../../source-rs/distribution/matrix/service/children/configuration.rs)
  reads its own `Matrix` configuration, resolves the launcher path in its realm,
  and installs child budgets and placement within the supplied ceiling. Startup
  requires empty slots. The distribution launcher now has a bounded editor for
  saved strip width and outer spacing, using its supplied user-realm grant.
  There is no live reconfiguration yet; editing saved bytes does not alter
  running children.
- Warden, encrypted realms and authentication are not implemented in the Rust
  reference OS. The current user picker selects an unencrypted subtree.

The broader design owners are [components](../../design/part_2_components/00_component_model.md),
[configuration](../../design/part_4_storage/02_configuration_and_policy.md),
[secrets](../../design/part_1_authority/04_secrets_and_keys.md) and
[sessions](../../design/part_1_authority/07_sessions_and_login.md).
