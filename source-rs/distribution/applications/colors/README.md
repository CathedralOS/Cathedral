# Colors

Start at [main.rs](main.rs), then [session/session.rs](session/session.rs).
The installed application receives local matrix interaction and storage links,
paints its scene, persists its color on Enter and handles its own close button.
The session delegates document state, retained surfaces and close interaction to
named siblings. It has no provider startup or desktop-owner responsibilities.

The library also supplies the same event loop to the
[recursive desktop lab](../../desktop/lab/desktop.rs). That composition selects
`run::<true>` and supplies its control-command handler; the installed executable
selects `run::<false>`, so probe dispatch and adversarial scenes are eliminated
from its build. The lab owns proof commands, while Colors owns the application
state and rendering exercised by those proofs.

The [distribution profile](../../profile.json) installs this executable as the
template's Colors program. Existing stock copies are refreshed through the
ordinary installer upgrade, preserving object identity and saved color data.
