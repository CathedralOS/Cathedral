# Cathedral initial program

Owns startup and restart policy for the built-in distribution. Boot supplies
four independent launch grants: display, input, the status application and
storage. Providers are ready before the app starts.
Init retains private provider control connections and checks health with bounded
requests. It replaces failed providers independently, reports their generations
to the application, and replaces an exited or unresponsive application without
restarting its providers. Scene state and drawing live in `applications/status`.

Main enters `session.rs`, which starts ready providers and the application,
then enters `session/supervision.rs`. Each provider's health protocol lives under
`session/providers/`; `session/child.rs` owns one child's lifetime and accepted
control connection. The [composition map](../README.md) names the executable peers. Exhausted provider retries end init
and reclaim its children. Normal builds have no fault injection. `recovery-lab`
adds explicit test requests and asserts that unrelated task identities survive.

This is bounded lab policy, not a production service manager or trusted recovery
path. A fork can replace init and its profile, including using no platform services.

The session also starts the independent counter app. `session/counter.rs` checks
its private storage-backed health and replaces only that child on failure. The
stock composition has five children (display, input, status, storage, counter).
