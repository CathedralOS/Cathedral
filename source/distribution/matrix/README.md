# Stock matrix behavior in Omega

Start at [service/main.omg](service/main.omg): it chooses work from named commands,
current facts and the authenticated event kind. [progress.omg](service/progress.omg)
shows what follows each completion. The [service boundary](service/README.md)
connects these operations to the Rust host that still owns connections and tasks.

The service is one package. Descend by responsibility, following the Rust tree:

- [requests](service/requests/main.omg): route from authenticated caller facts,
  select the request family and decide the consequences of owner operations.
- [children/control](service/children/control/main.omg): aggregate loss answers
  and decide whether an ordinary close can stop a child.
- [children/launch](service/children/launch/main.omg): read, admit, bind, place,
  activate and publish a child; recover resources after partial failure.
- [children/workspace](service/children/workspace/main.omg): catalogue instances,
  switch selection and restore the scene after cancellation.
- [children/startup](service/children/startup/main.omg): local picker/background
  retries; [realm](service/children/startup/realm/main.omg) discovers and configures
  the realm, picker and background.
- [children/recovery](service/children/recovery/main.omg): launcher crash/retry and
  fallback policy, including asynchronous read and admission completion.
- [children/configuration](service/children/configuration/main.omg): saved settings
  and live layout; [launch resources](service/children/launch/resources/main.omg)
  selects stock role offers and profile ceilings.

[bridge/policy/main.omg](bridge/policy/main.omg) is the temporary native dispatcher;
[bridge/native/main.omg](bridge/native/main.omg) selects its compiled export for
Rust. Service-turn, close/loss, startup, launcher recovery, launch workflow/roles
and resource adapters serialize named domain values there. The
[launch bridge](bridge/policy/launch/main.omg) also owns admission/import encoding
and image/declaration codecs; launch policy uses named facts and shared contract
values. The [workspace bridge](bridge/policy/workspace/main.omg) translates the
instance catalogue and switch workflow to named domain values as well. Requests
and realm configuration still expose packet-shaped interfaces and need the same
cleanup;
moving them into semantic folders does not finish that work.

Rust retains exact task/endpoint identity, authenticated connections, mappings,
late replies, yielding transport and resource reclamation. Common import-record
validation lives in [platform contracts](../../platform/contracts/imports/README.md).
The chooser imports the distribution configuration codec directly. Neither this
bridge nor the stock matrix policy is a new platform API.
