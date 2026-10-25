# Stock background

[main.rs](main.rs) acquires the scoped service bootstrap and enters
[backdrop.rs](backdrop.rs), which fills one owner-granted surface on geometry
changes and otherwise waits for notifications/control. It is a separate process,
not compositor code. The matrix resolves the `Background` executable in its own
realm, so replacing that object changes the backdrop without changing the kernel,
platform compositor or chrome. No storage or keyboard service is granted.

The process selects the shared Services transport before acquisition. It requests
storage once after its first interaction poll and requires refusal, then continues
drawing through the same broker. Its manifest and owner grant fund one notification,
one private mailbox page and one accepted reader page. Lifecycle controls and wake
hints still use copied channels.

The current implementation is a solid fill. Role selection UI, animation and
wallpaper storage grants remain distribution policy. Startup retry and fallback
belong to the owning matrix, not to this background process.
