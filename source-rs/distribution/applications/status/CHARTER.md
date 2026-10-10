# Cathedral status application

Owns the visible status scene, navigation, toggles and retained application state.
It runs in a separate address space and imports platform libraries and shared
contracts. Boot-approved display/input/storage links grant messaging, without launch,
cancellation, framebuffer mapping or raw keyboard/disk access. Init reports service
health and restart generations over the application's private control connection.

The app reconnects after provider replacement and redraws its retained state.
Its own replacement and a full reboot reload the saved private object. Each
selection/toggle is persisted before drawing; ambiguous writes are read back and
reconciled by generation before retry. Arrows select a panel and Enter
toggles it. Bitmap labels show ready provider generations and the last recovery.
This whole-screen demonstration is not trusted system chrome, a surface-isolated
application API or a desktop shell. The app owns the three-byte schema; platform owns the storage format.

Only `recovery-lab` includes F1/F2/F4 provider-fault requests to init, F3
crash/hang/backpressure probes, F5?F8 write-boundary pauses,
and probes proving that device and task-control calls are denied. Normal builds
cannot request provider replacement through the init protocol.
