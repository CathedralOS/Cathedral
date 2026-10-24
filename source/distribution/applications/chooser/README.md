# Chooser: Omega decisions in the reference desktop

The installed user picker and taskbar now use this Omega component for their
catalogue, tile geometry, selection and launch/close decisions. The executable
still starts in the [Rust session](../../../../source-rs/distribution/applications/chooser/session/session.rs).
That session obtains its matrix-supplied connections, reads scoped facts,
executes the returned intents and paints the existing desktop.

Start at [policy/main.omg](policy/main.omg). Its dispatch descends into:

- [catalogue](policy/catalogue/catalogue.omg): choose the hidden template or
  configured launcher, omit the background, limit visible entries and name a new
  user. [Metadata](policy/catalogue/metadata.omg) recognizes exact hierarchy names
  and executable media type, and prepares bounded display labels.
- [interaction](policy/interaction/interaction.omg): pointer/keyboard selection,
  copy/launch/settings/recovery intents, and exact observed instance selection
  before an ordinary close request.
- [layout](policy/layout.omg): shared tile geometry for painting and hit testing.
- [outcomes](policy/outcomes.omg): launch/copy failures, close refusal, receipt
  observation and the decision to proceed after selecting a task.

This is an incremental port. Settings editing and revision reconciliation,
force-stop confirmation, pointer edge tracking, rendering, IPC and the event
loop remain in Rust. The old Rust implementations of the migrated decisions
have been removed; probe feature sets use the same Omega decisions.

## Native boundary

[native/main.omg](native/main.omg) selects the policy as one independent
component. The shared [packaging tool](../../../../tools/omega-component/README.md)
checks and embeds its single relocation-free export. The
[Rust adapter](../../../../source-rs/distribution/applications/chooser/policy/policy.rs)
marshals a synchronous, exclusive `[u64; 128]` packet; Omega retains no state or
pointers. It gets no handles, addresses, PIDs or globally resolvable object IDs.
Object IDs and instance tickets come from this process's scoped connections.
The matrix still authorizes every requested operation.

This app-local decision interface is deliberately separate from the common
`omega-app` transport host used by Colors and Notes. It is not a new service,
an IPC protocol or the final Omega app startup ABI. It adds a 1-KiB call packet,
no private-page reservation and no extra IPC hop. A future session port can call
these decisions directly and remove the temporary Rust marshalling.

Word zero selects the decision; bounded inputs occupy words 1–79 and results
usually occupy 80–95. A return of one means a valid decision. Describe (0)
returns the packet size, six catalogue slots, four picker slots and 51 collected
executable entries, checked against the Rust contracts at startup. Other
operations are classify (1), catalogue (2), geometry (3), selection (4), intent
(5), outcome (6), task click (7), after selection (8), receipt observation (9)
and user name (10). Unknown operations and invalid catalogue/name requests
return zero. Metadata uses one byte per word at this temporary boundary.

## Remaining transport work

The chooser's storage input differs from Notes' `BULK`: `ENTRIES` returns an
immutable attachment containing up to 51 80-byte rows (object, revision and
64-byte metadata), plus a count and pagination cursor. The existing runtime
validates and releases one page at a time. The chooser gathers at most the first
51 executable entries while continuing the listing to resolve named roles.
`HEAD` and settings `READ` replies are inline; `COPY_TREE` attaches the new name;
settings `PUBLISH` attaches its 104-byte record. Launch, task, layout and recovery
replies are inline. No common-host attachment changes are needed for this step.

Before moving the remaining session to `omega-app`, add bounded `ENTRIES`
attachment handling there with conformance tests, then move settings/recovery
and rendering. Keep attachment custody in platform transport and catalogue/
configuration policy in this distribution. There is no new owner decision.

## Verification

`cargo test --locked -p cathedral-chooser --lib` runs nine Windows-hosted tests
against the actual compiled export: exact metadata, role resolution, catalogue
limits, rendering/hit-test agreement, keyboard/control boundaries, action
eligibility, exact-ticket close, refused operations and packet bounds. The
chooser is also in the default Rust host test set. QEMU evidence is recorded in
[the handoff](../../../../TASKS_RUST.md); settings and nested desktops must remain
part of verification as the rest of this program moves.
