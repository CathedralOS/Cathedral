# Inert ACPI topology extraction

Current stage: tested by Omega semantic evaluation. All 77 original scenarios and all three body-mutating negative controls pass, and the complete fixture source checks. Native execution and hardware integration have not run.

## Pin, license and scope

Modified MIT OR Apache-2.0 translation of `rust-osdev/acpi` 6.1.1 at `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, retaining the provenance and notices in [PORT.md](PORT.md). The pure topology algorithms build on [checked fixed tables](fixed.PORT.md). Original synthetic fixtures contain no captured firmware or externally licensed AML examples.

ACPI-003's CPU, interrupt-controller, timer and PCI-region descriptions are covered through explicit inputs and ordered outputs. NUMA/SRAT/SLIT extraction is not part of this milestone; those full-inventory anchors remain pending. No production boot path imports the package.

## Source map

[topology-inventory.json](topology-inventory.json) separately classifies all symbols in `src/platform/{mod,interrupt,pci}.rs`. `topology.omg` transforms a validated MADT entry into an inert typed fact. `timers.omg` describes a PM timer. `pci_regions.omg` decodes MCFG regions and performs checked numeric ECAM queries. The HPET description is the already translated `hpet.omg::hpet_info`; its fields and comparator-count correction are tested in the fixed-table suite.

Mapped `AcpiTables` constructors, live handlers, shared register objects, allocator-owned aggregate types, ACPI mode switching, event initialization and AP wakeup effects are explicitly omitted as whole APIs. Their presence in the pinned source is not a reason to fabricate authority. The pure record transformation is implemented; unported effects are not success stubs.

## Ordered bounded descriptions

`topology_header` provides the initial local APIC address, legacy-PIC flag and MADT revision. Starting with cursor 44, `topology_next` returns source/next byte offsets and one typed fact. Success advances or reports end; parser errors are propagated. The caller chooses storage and walks the stream to completion. Local APIC address overrides occur as ordered facts; the initial address is not mislabeled as a final resolved address.

CPU facts retain UID, APIC ID, APIC/x2APIC kind, enabled and online-capable flags. `ObservedBootApic` is explicit caller-supplied observed identity; unknown identity remains unknown. Parsing does not label a CPU Running or WaitingForSipi. ACPI 6.6§5.2.12.1 recommends putting the bootstrap processor first, but that firmware recommendation is separate from observed identity or live execution state. Table order remains intact even when the observed bootstrap CPU is later in the stream. This is a deliberate deviation from the pin's first-entry state inference.

APIC I/O controllers, interrupt overrides, global/local NMIs, address overrides and wake-mailbox addresses become typed facts. Interrupt overrides require ISA bus 0; interrupt polarity, trigger encodings and local interrupt lines are checked. Local NMI facts also retain polarity/trigger data that the pinned aggregate drops. GIC CPU/distributor/MSI/redistributor/translation entries retain their complete typed decoded fields. SAPIC and unknown entries remain in `Other` with their original typed/raw description. The extractor neither silently drops unknown entries nor infers that a mixed controller stream forms a usable interrupt configuration. Cross-entry uniqueness, model compatibility, and interrupt-controller activation are consumer policy; no full validated aggregate model is claimed.

PM timer descriptions preserve 24/32-bit capability and selected GAS facts. The primary hardware-reduced flag suppresses the legacy PM timer before inspecting ignored legacy descriptors. HPET remains a separate inert description, including unknown address-space data; neither timer descriptor authorizes register access.

`pci_region` returns each checked MCFG record. `find_address` accepts an initialized 253-slot array and explicit count (the maximum number of whole 16-byte records within this package's 4096-byte table profile). Regions in the same segment may cover disjoint bus intervals. Multiple matching regions fail explicitly instead of selecting the first. Invalid device/function numbers, inverted bus ranges and u64 arithmetic overflow fail. No memory read, mapping, resource reservation or firmware method invocation follows from a numeric address.

## Primary facts and deviations

[ACPI 6.6 chapter5](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html) supplies MADT ordering/entry facts and FADT hardware-reduced behavior. The [PCI-SIG multiple-base-address ECN summary](https://pcisig.com/PCIFirmware/ECN/Firmware/EnablingMultipleBaseAddressesperPCISegmentGroup) confirms multiple ranges per segment; its full document requires member login and was not retrieved. The [Linux PCI maintainers' primary ACPI documentation](https://raw.githubusercontent.com/torvalds/linux/master/Documentation/PCI/acpi-info.rst) explicitly states that MCFG/_CBA base addresses correspond to bus 0 even when the managed bus range starts later. Only that numeric addressing fact is used; no Linux implementation or expressive source text was copied.

Accordingly `physical_address` computes the function offset with the actual bus number, correcting the pin's subtraction of the first managed bus. Device/function fields are bounded to 31/7, additions checked, and conflicting matches produce error 37. No assumption about physical-address canonicality, mapped resources, alignment approval or access rights is added.

The source's allocator vectors become bounded explicit inputs or an ordered result stream, not a silently truncated fixed aggregate. `ProcessorState` is replaced by declared firmware flags plus separate observed identity. These representation/API changes are deliberate and listed in the source map.

## Errors and verification

Inherited fixed-table errors retain their meanings. Additional errors are 32 invalid interrupt-override bus,33 invalid local NMI line,34 invalid PCI device,35 invalid PCI function,36 numeric address overflow,37 ambiguous PCI bus-range match. `found=false,error=0` means no PCI region matched; an empty region set does not bypass device/function validation.

The three sliced upstream files contain no `#[test]` functions. The 77 original fixtures cover enabled/online-capable/boot identity, APIC and GIC typed facts, source order and unknown retention, malformed input, NMI validation, hardware-reduced timers, bus 0-relative ECAM arithmetic, full-width boundaries and overlapping/disjoint ranges. Body-mutating negative controls change CPU enabled state, nonzero-start-bus ECAM address and malformed MADT acceptance. They must fail through computed `1 == 0`, proving execution of the actual Omega expectation bodies.

Compiler: clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`; `/tmp/cathedral-omega-eaa7993/release/omega`, SHA-256 `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/acpi/topology_check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
python3 tools/ports/acpi/topology_evidence.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi --require-transcribed source/libraries/acpi/topology-inventory.json
```

This evidence is semantic evaluation of pure machines. No Omega native ABI layout, firmware mapping, MMIO, port access, interrupt configuration, CPU activation or hardware timing result is claimed.
