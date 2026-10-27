# Chooser decisions

These are ordinary Omega operations, used directly by the installed chooser.
They mirror the responsibilities of Rust's `chooser/policy/`, without carrying
its temporary native-call encoding into the domain model.

- [Catalogue](catalogue/catalogue.omg) resolves the hidden template/launcher,
  excludes the background and bounds the visible entries. [Metadata](catalogue/metadata.omg)
  recognizes exact roles and executable type; [name](catalogue/name.omg) names a new user.
- [Interaction](interaction/interaction.omg) dispatches a typed input to
  [pointer hit testing](interaction/pointer.omg) or [keyboard navigation](interaction/keyboard.omg).
  Both share the [geometry](layout.omg) used by the painter.
- [Actions](interaction/actions.omg) turns a selection into a launch, copy,
  settings, management or retry intent. [Task hits](interaction/tasks.omg)
  select an observed task ticket before any close request.
- [Outcomes](outcomes.omg) interprets actual replies, including refusal,
  lost work, launch receipts and stopped-task observations.

[The app session](../app/session/session.omg) owns service sequencing and
persistent UI state. [Its input entrance](../app/session/input.omg) decodes
Matrix events and tracks scoped pointer edges before invoking interaction policy.
[Its action entrance](../app/session/actions.omg) executes the selected intent
through supplied endpoints. A UI selection grants no authority.

Rust probes enter through [the native adapter](../bridge/policy/main.omg).
Only that adapter knows the 128-word policy packet and operation numbers.
The ordinary app has no dependency on that package.
