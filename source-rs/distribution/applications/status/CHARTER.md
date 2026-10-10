# Cathedral status application

Owns the visible status scene, navigation, toggles and retained application state.
It runs in a separate address space and imports platform libraries and shared
contracts. Boot-approved display/input links grant messaging, without launch,
cancellation, framebuffer mapping or raw keyboard access. Init reports service
health and restart generations over the application's private control connection.

The app reconnects after provider replacement and redraws its retained state.
Its own replacement starts with fresh state. Arrows select a panel and Enter
toggles it. Bitmap labels show ready provider generations and the last recovery.
This whole-screen demonstration is not trusted system chrome, a surface-isolated
application API or a desktop shell. There is no persistence or storage yet.

Only `recovery-lab` includes F1/F2 provider-fault requests to init, F3 crash/hang/backpressure probes,
and probes proving that device and task-control calls are denied. Normal builds
cannot request provider replacement through the init protocol.
