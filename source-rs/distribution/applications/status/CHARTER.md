# Cathedral status application

Owns the visible status scene, navigation, toggles and retained application state.
It runs in a separate address space and imports platform libraries and shared
contracts. Boot-approved display/input/storage links grant messaging, without launch,
cancellation, framebuffer mapping or raw keyboard/disk access. Init reports service
health and restart generations over the application's private control connection.

The app reconnects after provider replacement and redraws its retained state.
Its own replacement and a full reboot reload the two saved private records. Each
selection/toggle is persisted before drawing; ambiguous writes are read back and
reconciled by generation before retry. Arrows select a panel and Enter
toggles it. Bitmap labels show ready provider generations and the last recovery.
This whole-screen demonstration is not trusted system chrome, a surface-isolated
application API or a desktop shell. The app owns its record schemas; platform owns the catalog and storage format.

Only `recovery-lab` includes F1/F2/F4 provider-fault requests to init, F3
crash/hang/backpressure probes, F5-F8 write-boundary pauses,
and probes proving that device and task-control calls are denied. Normal builds
cannot request provider replacement through the init protocol.

Start at `main.rs`, then `application.rs`: restore, receive, update, save, render.
The `application/` children own connections, input, state/schema, persistence and
view. Rendering reports completed frames through `view/report.rs`; optional
failure probes stay in `application/lab.rs`. The
[composition map](../../README.md) identifies the other executable entrances.

Selection and toggle bits are separate records, committed together with matching
root revision tags. The app lists them in the APPLICATION panel after a successful
save. Legacy scene records remain readable and convert on the next user edit.
The independent counter app uses the same object numbers in a different root.
