# Stock matrix configuration

The chooser and stock matrix share this distribution-owned Omega package.
Start at [main.omg](main.omg): saved bytes go through [encoding.omg](encoding.omg)
and canonical [name.omg](name.omg); geometry goes to [placement.omg](placement.omg).
This is the desktop's CTHMAT02 format and bottom-bar policy, not a universal
platform configuration or a source of authority. The owner still checks the
supplied grants before applying any result.

The chooser calls the codec directly. The Rust stock matrix calls the compiled
export through [its policy entrance](../../../../source-rs/distribution/matrix/policy/policy.rs).
[configuration-native](../configuration-native/main.omg) packages that temporary
synchronous bridge: 128 u64 words; operation 1 normalizes 104 bytes stored one per
word at 16, operation 2 takes margin/bar/gap/width/height in words 1-5 and returns
two rectangles in 16-23. Success is 1, refusal 0. This is not an IPC wire format.

Rust's public data representation remains in the distribution contracts for
installer/tools and serves as the independent compatibility oracle. Compiled
conformance tests compare every single-byte mutation, resource and legacy bounds,
UTF-8 names, and 100,000 placement combinations including u64 overflow boundaries.
The default Rust host test command includes them on Windows. Other host targets
must not silently substitute Rust behavior for the native export.

This is the first stock matrix policy slice in Omega. Admission, supervision,
IPC dispatch and upstream rendering/storage adaptation are still Rust.
