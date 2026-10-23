# Components and matrices

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-23 |

## Scope

This contract establishes matrix policy ownership and supplied authority,
following the owner's 2026-10-18 clarification. It does not specify a universal
matrix configuration format, the complete component lifecycle, or realm
encryption and locking protocols.

## A matrix is an ordinary program

A matrix is a program acting as a host and attenuator for children. Its parent
determines the authority available to it; the matrix determines what it supplies
to its children within that authority. Hosting a matrix uses the same authority
boundary as hosting another program.

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

The owner-approved model supplies one matrix-scoped service broker at application
startup. A manifest declares required/optional imports; the launching owner
resolves the authority ceiling before launch. The application requests its service
connections through that broker and retains them for ordinary operations. A new
service need not change main's argument structure. The broker authenticates the
connection, never an application-supplied identity, and cannot widen its approved
ceiling. Returned connections preserve the matrix's chosen mediation and revocation.
Bootstrap and service protocol versions evolve independently.

The Rust [bootstrap contract](../../../source-rs/platform/contracts/services/README.md)
records the first bounded implementation and its compatibility path. Direct
startup grants remain necessary for the privileged programs establishing brokers;
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
