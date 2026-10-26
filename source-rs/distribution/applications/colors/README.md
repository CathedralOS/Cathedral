# Colors

Start at [main.rs](main.rs), then the
[Omega app](../../../../source/distribution/applications/colors/README.md).
The installed application owns its behavior in Omega, using the common
[native host](../../../platform/libraries/omega-app/README.md) for scoped
service acquisition, transport and buffer custody. The startup import record
and executable adapter remain Rust. Normal F5 builds the checked-in source.

The retained Rust [session/session.rs](session/session.rs) supplies probes to the
[recursive desktop lab](../../desktop/lab/desktop.rs). That composition selects
`run::<true>` and supplies its control-command handler. It is not the installed
app's implementation. Its interaction stream is owned by `Services`; all drawing
commands and image attachments use that same shared connection. A separate shared
storage client first proves wrong-service refusal, then transfers unchanged into
`Scope`; binding discovery and writes continue its request sequence. Both ends
are explicitly funded by the lab. Positional input hints still use the older wake
queue until that startup shape is migrated to broker acquisition.

The ordinary desktop and native component tests exercise
the Omega implementation; kernel labs retain their existing proof commands.

The [distribution profile](../../profile.json) installs this executable as the
template's Colors program. Existing stock copies are refreshed through the
ordinary installer upgrade, preserving object identity and saved color data.
