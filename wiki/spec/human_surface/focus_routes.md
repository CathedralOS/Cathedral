# Focus groups and recursive routes

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-23 |

## Scope and ownership

The owner accepted this topology and transition baseline on 2026-10-23. It builds
on [input delivery](input.md) and [logical channels](input_channels.md). Platform
protocols provide scoped registration, routing and transition mechanisms; matrix
programs choose local selection policy. The kernel supplies generic isolation,
IPC and scheduling, not window cycling or application focus policy.

A focus group associates authorized channels with a shared local selection. A
focus route is the chain of selected direct children formed across matrices.
Neither is a global user identity, a view, a cursor image or a kernel IPC channel.
The system supports a funded number of groups without privileged human/agent
classes. Apps do not need to construct a new input interface for each group.

## Registration and exclusive control

Registration requires a grant from the supplying matrix. Creating a group returns
private authority to control that group's local selection, bind permitted
channels and unregister it. Creation does not grant authority over another group
or over sources, destinations or presentations outside existing grants.
Knowing an identifier grants no control or observation rights.

Each group has one controlling owner. Independently shared control is not part
of this baseline. Cursor and shortcut handlers may send requests to that owner,
which applies its policy; they do not acquire control by sending requests or
rendering a cursor. The enclosing matrix retains supervisory authority within
its grants, including constraining or terminating the group.

The owner serializes accepted selection changes and input routing. A request to
change selection is not proof that it took effect. Input dependent on that change
must be ordered after its accepted transition or tied to the intended valid
selection; delayed requests must not silently redirect input intended for an
earlier target. The acknowledgement and ordering wire mechanism is not frozen.

Unregistration retires the binding and resolves outstanding interactions. Reused
storage or delayed requests cannot control a replacement group. Owner failure
follows the [subtree lifetime](../execution/components_and_matrices.md#matrix-failure-and-subtree-lifetime)
and IPC custody rules, not orphaned route ownership.

## Local selection and recursion

Each matrix maintains local selections of direct children, or no child, for its
groups. Selecting a child matrix permits input to continue through that matrix's
own routing. The child cannot select itself in its parent; the parent's local
group control does not imply access to the child's internal selection controls.
Explicit supervisory grants remain separate.

Children receive scoped channels and authorized routing associations, not the
complete ancestor route, global group identities or sibling state. Associations
across a boundary are explicit, not inferred from matching names or identifiers.

Local selections persist when an ancestor stops forwarding. The broken upstream
route does not clear a child's selection or require a global inactive-focus
state. The child may still receive input from independent authorized sources.
When forwarding resumes, retained local selections apply if their targets and
grants remain valid. Silence alone conveys no focus or authority change.

Ending an incoming interaction is distinct from changing local selection. Cleanup
of a discontinued stream propagates to its previous recipients as necessary
without erasing unrelated groups, independent sources or remembered selections.

## Targeting and presentation are distinct

Channels can cooperate without every event using identical targeting:

- Keyboard-like input follows the group's selected route.
- Pointer motion uses that pointer's position and permitted hit targets.
- An established drag follows that pointer's capture.
- Controller input may follow selection or an explicitly authorized fixed target.

A keyboard and pointer may share selection while an agent has its own group.
Several pointers may share selection but retain separate positions and captures.
Cursor rendering alone grants no submission, selection or capture authority.

The router resolves clicks using authorized scene/hit-test information. Local
policy chooses select-only or select-and-deliver behavior. The selecting gesture's
target is fixed when routed; later movement or replacement cannot retarget it.
Forwarding into a child matrix continues the gesture rather than generating a
duplicate press. Consumed selection gestures must not leave an unmatched release
or activate a recipient that never received their press.

Changing keyboard selection alone need not cancel independently authorized
pointer capture. Capture remains bounded by its grant and target lifetime. If
its forwarding path is withdrawn, its outstanding interaction must be resolved
rather than stranded or transferred to a new target.

Selection does not grant restore, raise, stacking, visibility or capture rights.
Stock chrome may combine separately authorized presentation and selection changes.
The [presentation contract](rendering.md) separates rendering demand from input
authority; precise hidden-target and scene/hit-test rules remain separate.

An app may be selected by several groups simultaneously. Selection notifications,
where exposed, identify the relevant scoped association rather than only a single
application-wide boolean. Apps still own channel-to-role assignment and may use
shared state, independent carets/players or defined exclusive-control behavior.
Independent routing does not promise independent application state.

## Selection changes and outstanding input

Standing routing grants are separate from the currently selected recipient.
Switching selection does not mint general authority over the newly selected app.

When a switch interrupts an interaction, cancel affected events still owned by
the router and publish an ordered loss/cancellation boundary to the old recipient.
Resolve held state there; suppress already-held controls until release before
they can activate the new recipient. A release completes an old interaction, not
a fresh action at the new target. Independent source contributions remain intact.

The boundary is scoped to the affected stream/association. A nested matrix can
propagate cancellation while retaining its selections. Cancellation is protocol
state, not a new click, permission to select elsewhere, or rollback of executed
work. A failed recipient need not acknowledge before the owner stops future
delivery; cleanup must still preserve memory and transport custody.

Events already published into IPC may still be consumed and cannot be assumed
retractable. Router-owned events and published messages are different commit
stages. Concrete IPC enqueue/closure and shared-ring publication rules must define
that boundary and ordering before their APIs are frozen. There is no universal
one-event exposure limit. This does not resolve general IPC revocation by
asserting that delivered work can be undone.

## Evidence and limits

The Rust [local router](../../../source-rs/platform/libraries/matrix/world/input/input.rs)
and [child queues](../../../source-rs/platform/libraries/matrix/world/children.rs)
now exercise separate targeting, cancellation and held-key suppression for four
bounded upstream streams. Host proofs cover independent selection, scoped overflow,
fair draining and recursive cancellation without losing the next fresh press.
Explicit associations now allow several channels to share a group's selection
while keeping independent capture. Group-local delivery order survives recursive
forwarding; keyboard-only cancellation does not end a pointer's drag. Host proofs
cover those transitions and refusal to silently reassociate an existing stream.
The normal desktop uses distinct pointer and keyboard channels in one group.
The owner control endpoint can now create/release groups, register/retire channels
and select a local target. Group retirement follows channel retirement; replacing
the last channel alone preserves remembered selection through nested routers.
Ordinary child connections cannot invoke these operations. General delegated
control grants, app producer acquisition, live reassociation and all capture
semantics in this contract remain unimplemented.

Current recursive event forwarding remains an implementation baseline, not a
mandated production transport. The host routing proofs do not establish a complete
multi-group desktop or producer registration service.
Wire schemas, group limits and transition acknowledgements remain work.
[Placement and rendering demand](rendering.md) have a separate partial contract.
That contract also defines accepted-layout hit-testing and pointer-target cursor
preferences independently of keyboard selection. Exact mask publication,
protected indicators and protected-prompt protocols are not selected here.
