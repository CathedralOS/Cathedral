# Stock background

[main.rs](main.rs) acquires the scoped service bootstrap and enters
[backdrop.rs](backdrop.rs), which fills one owner-granted surface on geometry
changes and otherwise waits for notifications/control. It is a separate process,
not compositor code. The matrix resolves the `Background` executable in its own
realm, so replacing that object changes the backdrop without changing the kernel,
platform compositor or chrome. No storage or keyboard service is granted.

The current implementation is a solid fill. Role selection UI, animation, wallpaper
storage grants and automatic background restart are future distribution policy.
