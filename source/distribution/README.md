# Stock distribution

[Matrix service](matrix/service/main.omg) chooses the next unit of desktop work.
Its child responsibilities live under `matrix/service/children/`, following the
Rust service. [The composition map](matrix/README.md) identifies the host mechanisms
that still execute in Rust and the temporary native bridge.

[Applications](applications/) contains the chooser, Notes, Colors and
[Background](applications/background/README.md). Each app's
`app/main.omg` exposes its compiled entry and descends into its session and UI
behavior. All four install standalone `native/` executables;
`process/` owns their service communication and app execution, while `app/` owns
behavior. Their retained `bridge/` packages support explicit comparison tests.

[Libraries](libraries/) contains distribution-owned UI code, including the vendored
Omega UI library. [Lab](lab/) holds independent composition experiments. Common
service protocols and reusable non-distribution behavior belong in `platform/`.

`profile.json` selects the separate early Omega kernel experiment. Normal desktop
composition still comes from the [Rust distribution](../../source-rs/distribution/README.md).
