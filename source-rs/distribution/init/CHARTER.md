# Cathedral initial program

Owns startup and restart policy for the built-in distribution. Boot supplies
four independent launch grants: display, input, the status application and
storage. Providers are ready before the app starts.
Init retains private provider control connections and checks health with bounded
requests. It replaces failed providers independently, reports their generations
to the application, and replaces an exited or unresponsive application without
restarting its providers. Scene state and drawing live in `applications/status`.

Main orchestrates startup through `supervisor`; `service` owns one child's
lifetime and accepted control connection. Exhausted provider retries end init
and reclaim its children. Normal builds have no fault injection. `recovery-lab`
adds explicit test requests and asserts that unrelated task identities survive.

This is bounded lab policy, not a production service manager or trusted recovery
path. A fork can replace init and its profile, including using no platform services.
