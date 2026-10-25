# Omega application host for the Rust reference OS

Start at [host/host.rs](host/host.rs): obtain the app's requested scoped
connections, dispatch one operation, and resume the Omega app with its result.
[exchange.rs](host/exchange.rs) owns request serialization and frozen attachment
custody; [attachments.rs](host/attachments.rs) stages larger outgoing objects.
[transfer.rs](transfer.rs) checks byte extents before allocation/mapping.
[component.rs](component.rs) checks the native layout and borrows the
opaque state. There is no Colors state, app schema, rendering policy or input
decision in this package.

The producer-side helpers live at
[app-host/main.omg](../../../../source/platform/libraries/app-host/main.omg).
Consumers are [Omega Colors](../../../../source/distribution/applications/colors/README.md)
and [Omega Notes](../../../../source/distribution/applications/notes/README.md),
plus the [Omega chooser](../../../../source/distribution/applications/chooser/README.md).
This is deliberately a temporary, synchronous x86-64 native bridge. A future
Omega runtime can replace it while retaining the app's protocol behavior.

Storage acquisition creates one shared-memory connection owned by the host.
Subsequent storage operations retain its request sequence and immutable attachment
custody. A failed or ambiguous exchange retires it without replay. The app's
manifest and owner grant must fund the notification, private mailbox page and
accepted reader page; optional storage remains optional. The host selects shared
broker transport before entering the app. Service acquisition and interaction use
one persistent `Services` client, including moved endpoint replies and immutable
images. Interaction acquisition supplies a separate notification wait right;
the matrix owns its signal/control rights. Mixed event waits consume input hints
independently of reply notifications and preserve control-first dispatch. Lifecycle
control reception uses [OwnerControl](../user-runtime/server/owner.rs), which
opens only the process's owner port and retains the exact received request until
operation 7 answers it. It discovers no service links and trusts no supplied PID.
Split selects shared control; stock owners still send copied lifecycle requests
until their migration. A disconnected owner control link leaves input usable.
See the
[transport boundary](../user-runtime/connection/README.md).

## ABI 1

One System V export accepts pointers to zeroed opaque state and a 4,096-word
`u64` packet. Packet word 15 selects state size (1), alignment (2), ABI version
(3), or an app step (0). Queries never touch state. The host reserves at most
4,096 state bytes aligned to eight and refuses incompatible layouts. Each call
borrows its buffers and retains no pointer after returning. The packet and state
use nine private pages charged to the app, outside its bounded stack.

During a step, the app returns the operation below. Words 0–7 hold a 64-byte
platform request or response; word 8 is the outgoing/returned attachment byte count;
words 16 onward hold at most 32,640 payload bytes. Word 9 returns transport
status (zero succeeds), separate from the provider status in word 0. Words
11 supplies the receive limit for a storage BULK or ENTRIES request. Word 12
contains the ordinary task argument on the first step. Word 13 optionally bounds
WAIT in monotonic ticks (zero waits indefinitely); word 14 is reserved.
The host clears the receive limit before resuming the app. No wire identifiers
grant authority by themselves.

| Operation | Host work |
| --- | --- |
| 0 | Exit normally |
| 1 | Acquire interaction, requested minimum/maximum version in words 0/1 |
| 2 | Acquire storage with the same version fields |
| 3 | Bind the granted storage scope; return `[0, realm, 0…]` |
| 4 | Storage request through `Scope`; retain its binding/deadline rules |
| 5 | Interaction exchange through the acquired pair |
| 6 | Wait for wake, owner control or the optional word-13 refresh deadline; word 9 is 0 for wake/refresh, 1 for control, otherwise a transport error; control bytes occupy 0–7, own CPU time occupies word 10 |
| 7 | Reply to the owner control message with words 0–7 |
| 8 | Write payload bytes to the app's existing diagnostic output |
| 9 | Append a payload chunk to a staged object; words 0/1 are total bytes and sequential byte offset |
| 10 | Discard the staged object, whether partial or ready |
| 11 | Read this task's existing monotonic clock; return ticks in word 0 |

Storage BULK receives require an explicit nonzero limit no larger than the packet
payload. Before mapping, the host checks the returned byte count against that
limit, the request length and the received object's size. It copies exactly those
bytes into the packet, returns their count in word 8 and releases its reader
custody. Page padding is never exposed. Unexpected attachments, malformed counts
and successful nonempty BULK replies without backing fail explicitly.
ENTRIES uses its count of 80-byte rows instead: at most 51, with canonical metadata,
strictly ascending object numbers after the requested cursor and the final number
matching the reply cursor. An empty listing carries no attachment. Only the
declared rows are copied; page padding is excluded. Other incoming-object
operations remain unsupported. The app still validates its schema
and expected revision; the host does not interpret document contents.

Larger outgoing images use one staged object of at most 64 KiB, allocated from
the app's private budget. Chunks must be nonempty, contiguous and within the fixed
total; overlapping, skipped or oversized chunks discard the transfer. Completion
freezes/unmaps the object. The next storage or interaction exchange consumes it
with word 8 zero. Partial publication and simultaneous inline/staged payloads
are refused and released. Allocation, freeze or exchange failures also release
custody; task exit releases any pending object. Notes' 46,080-byte text bitmap
uses two chunks without enlarging every app's permanent packet.

The host never chooses a service globally or supplies raw endpoint names to
Omega. Existing `Services` and `Scope` enforce supplied connections, version
checks, retirement after ambiguous exchanges, and recursive realm scoping.
There is one outstanding request per connection. Slow interaction replies are
awaited without retrying mutations. Storage failure may hide a commit; the app
must reconcile. Frozen outgoing objects survive until completion, then the host
releases its producer custody. Providers retain only their own accepted rights.

Owner control is checked before wake hints to prevent an input stream from
starving lifecycle questions. The app still decides its batching and when to
wait. The host consumes word 13 on entry; a refresh timeout returns an empty wake,
not an input event. The chooser uses it to observe asynchronous task collection
even after the last surface notification. Retiring the optional owner control link leaves wake/input service active;
it does not change the kernel's parent-death subtree teardown. This API does not
promise hard real-time scheduling or memory safety for
arbitrary native code. The ordinary process sandbox remains the trust boundary.
