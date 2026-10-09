# Method and device-status metadata

`object_metadata.omg` supplies two pure decoders. `method_flags(u8)` returns
argument count, serialization flag and synchronization level. `device_status(u64)`
returns the five independently observed status bits. Inputs are initialized numeric
facts; outputs grant no authority and perform no AML evaluation or I/O.

The source derives from public `MethodFlags` and `DeviceStatus` in rust-osdev/acpi
[`object.rs` at 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/object.rs),
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Cathedral retains the exact licenses
and notices under `licenses/rust-osdev/acpi` and `THIRD_PARTY_NOTICES.md`.
The selected inventory translates ten anchors; other Object operations remain
outside this slice. Ordinary decoded records replace Rust tuple wrappers and
individual getters without assigning a native representation to those wrappers.

[ACPI 6.6 §20.2.5.2](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html#named-objects-encoding)
defines the three MethodFlags bit fields. All byte values decode; a nonzero
synchronization level is retained even for a nonserialized method. The caller
implements synchronization policy. Existing method/frame consumers are unchanged.

[ACPI 6.6 §6.3.7](https://uefi.org/specs/ACPI/6.6/06_Device_Configuration.html#sta-device-status)
defines `_STA` bits. This helper reports bits without deciding whether firmware's
combination is valid: enabled while absent remains observable as such. Higher bits
are ignored by these five getters, matching the pin; their omission is not a claim
that firmware reserved bits may be nonzero. Battery applicability, absent `_STA`
defaults, enumeration and `_INI` ordering belong to a later policy adapter.
`functioning=true` does not imply `present=true`. No cached runtime status is added.

The evidence suite compares actual public Rust getter observations with actual
Omega body results for all 256 MethodFlags bytes and all 32 low status combinations
under six higher-bit patterns (zero, bit5, bit31, bit32, bit63, and all higher bits).
Those 448 observations are grouped into 56 behavior/control pairs. Each control
changes one expected field inside the executed body. A representative constant
pair includes maximal method flags and `u64::MAX` status. Exact source/tool hashes
and outcomes are retained in `tools/ports/acpi/aml/object-metadata/verification.json`.
These checks do not establish native ABI, runtime synchronization or hardware behavior.
