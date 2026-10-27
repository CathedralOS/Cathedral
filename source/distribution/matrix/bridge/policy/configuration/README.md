# Native configuration adapter

[main.omg](main.omg) dispatches the temporary Rust call envelope to
[bytes.omg](bytes.omg) or [layout.omg](layout.omg). The service owns the actual
[saved format](../../../service/children/configuration/encoding.omg),
[IPC syntax](../../../service/children/configuration/wire.omg) and
[layout decisions](../../../service/children/configuration/main.omg).

The native envelope has 128 u64 words. Operation 1 normalizes 104 file bytes
stored one per word starting at 16. Operation 2 takes margin/bar/gap/width/height
in words 1-5 and returns chrome/workspace rectangles in 16-23. Both return 1 on
success and 0 on refusal.

Operation 18 accepts host-authenticated authority/busy facts in 2-3, extent in
4-5, standing positions in 6 and the original IPC words in 16-23. It returns a
native status, with accepted chrome/app/background rectangles in 32-51 and
remembered app positions in 52. Exact caller, epoch and resource custody stay
with the Rust host. This envelope is a migration detail, not another IPC protocol.
