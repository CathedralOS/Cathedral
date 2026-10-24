# Omega application host for the Rust reference OS

Start at [host/host.rs](host/host.rs): obtain the app's requested scoped
connections, dispatch one operation, and resume the Omega app with its result.
[exchange.rs](host/exchange.rs) owns request serialization and frozen attachment
custody. [component.rs](component.rs) checks the native layout and borrows the
opaque state. There is no Colors state, app schema, rendering policy or input
decision in this package.

The producer-side helpers live at
[app-host/main.omg](../../../../source/platform/libraries/app-host/main.omg).
The first consumer is [Omega Colors](../../../../source/distribution/applications/colors/README.md).
This is deliberately a temporary, synchronous x86-64 native bridge. A future
Omega runtime can replace it while retaining the app's protocol behavior.

## ABI 1

One System V export accepts pointers to zeroed opaque state and a 4,096-word
`u64` packet. Packet word 15 selects state size (1), alignment (2), ABI version
(3), or an app step (0). Queries never touch state. The host reserves at most
4,096 state bytes aligned to eight and refuses incompatible layouts. Each call
borrows its buffers and retains no pointer after returning. The packet and state
use nine private pages charged to the app, outside its bounded stack.

During a step, the app returns the operation below. Words 0–7 hold a 64-byte
platform request or response; word 8 is the outgoing attachment byte count;
words 16 onward hold at most 32,640 payload bytes. Word 9 returns transport
status (zero succeeds), separate from the provider status in word 0. Words
11–14 are reserved. No wire identifiers grant authority by themselves.

| Operation | Host work |
| --- | --- |
| 0 | Exit normally |
| 1 | Acquire interaction, requested minimum/maximum version in words 0/1 |
| 2 | Acquire storage with the same version fields |
| 3 | Bind the granted storage scope; return `[0, realm, 0…]` |
| 4 | Storage request through `Scope`; retain its binding/deadline rules |
| 5 | Interaction exchange through the acquired pair |
| 6 | Wait for wake or owner control; word 9 is 0 for wake, 1 for control, otherwise a transport error; control bytes occupy 0–7, own CPU time occupies word 10 |
| 7 | Reply to the owner control message with words 0–7 |
| 8 | Write payload bytes to the app's existing diagnostic output |

This first host supports outgoing attachments and inline replies. An incoming
attachment is released and reported as unsupported, never silently accepted.
Notes' bulk reads will require an explicit bounded incoming-object binding.

The host never chooses a service globally or supplies raw endpoint names to
Omega. Existing `Services` and `Scope` enforce supplied connections, version
checks, retirement after ambiguous exchanges, and recursive realm scoping.
There is one outstanding request per connection. Slow interaction replies are
awaited without retrying mutations. Storage failure may hide a commit; the app
must reconcile. Frozen outgoing objects survive until completion, then the host
releases its producer custody. Providers retain only their own accepted rights.

Owner control is checked before wake hints to prevent an input stream from
starving lifecycle questions. The app still decides its batching and when to
wait. This API does not promise hard real-time scheduling or memory safety for
arbitrary native code. The ordinary process sandbox remains the trust boundary.
