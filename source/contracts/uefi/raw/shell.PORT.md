# Shell protocol remainder (UEFI-009)

Stage: **typechecked raw declarations**; the complete named ShellProtocol layout
plan is transcribed but blocked by the current 32-field reflection capacity.
This closes the shell protocol file outside the earlier shell-parameter slice.

Modified translation of [rust-osdev/uefi-rs](https://github.com/rust-osdev/uefi-rs)
at `c0facddf9ba42b74906a37fca2869e6cdbc8da6a`, preserving **MIT OR Apache-2.0**.
Copyright (c) The uefi-rs contributors. See repository
[notices](../../../../THIRD_PARTY_NOTICES.md) and retained licenses.

The [inventory](shell-inventory.json) maps all 61 lexical anchors in
`uefi-raw/src/protocol/shell.rs` to [shell_protocols.omg](shell_protocols.omg):
ListEntry, ShellFileInfo, ShellDeviceNameFlags, the full 46-field ShellProtocol,
two flag values and its GUID. All fields and original native signatures remain
explicit. There are no omitted or pending symbols, standalone algorithms or
upstream test bodies. Rust derive/bitflags ergonomics are intentionally absent.

The governing primary source is the [UEFI Shell Specification 2.2](https://uefi.org/sites/default/files/resources/UEFI_Shell_2_2.pdf),
EFI_SHELL_PROTOCOL interface and file-list declarations. The source pin defines
the exact subset; later shell revisions are not inferred. Omega's layout plans,
foreign-storage and boundary-shape documents were consulted for this corpus.

Status uses an inert x64 u64 here; event/file handles and all pointer/function
slots use inert addr. ShellFileInfo preserves its nested list entry. These are
foreign representation carriers, not the validated handles in Cathedral's
boot-facing contracts. No command executes, environment changes, file access,
linked-list dereference or native callback occurs in this package.

[The 65 vectors](shell.vectors.json) include all record sizes/alignments/offsets,
the two flags and the GUID bytes. A locked Rust probe cross-compiles actual
upstream declarations for x86_64-unknown-uefi. The shared exact-pin tooling also
compares source field metadata/order/types and authored plan geometry.

```sh
python3 tools/ports/uefi-machine/check.py --slice shell
../Omega/target/release/omega --check source/contracts/uefi/raw/shell_protocols.omg
../Omega/target/release/omega --check source/contracts/uefi/raw/shell_layouts.omg
```

The first two checks pass. The third produces exactly the 14 unprovable array
indices 32–45 because core layout Schema.fields contains 32 elements. The full
named shape is retained, with `PORT-BLOCKED[omega:layout-reflection-capacity]`;
no array-of-slots replacement hides the issue. These observations use the local
release binary SHA-256
`a87e533bc7c068419ab2ff05c213abbb4a874e3fe8dbba35e5636004e6f5bfe6`, whose build
revision is not independently established. Native calling-policy/address
materialization remains a separate recorded seam. Emitted Omega ABI comparison,
firmware execution, and Cathedral integration are not run. No production build
imports these declarations.
