"""Reviewed module-level reconciliation, not inferred semantic translation status."""
F='pure-fact-layout'; A='pure-algorithm'; E='existing-representation'; I='instruction-boundary'; P='cathedral-policy'; R='deliberate-rejection'; T='test-fixture'
FACT='source/drivers/facts/'
CORE='source/core/'
def ref(path,anchor):return {'path':path,'anchor':anchor}
PTE=ref(FACT+'x86_page_table_entry.omg','pub data X86PageTableEntry')
IDT=ref(FACT+'x86_idt_gate.omg','pub data X86IdtGate')
EXC=ref(FACT+'x86_exception_vectors.omg','pub const X86_EXCEPTION_VECTOR_COUNT')
IST=ref(FACT+'x86_interrupt_stacks.omg','pub data X86IstStackClass')
WALK=ref(CORE+'x86_page_table.omg','pub machine x86_validate_bootstrap_four_level_walk_descriptor')
ADDR=ref(CORE+'x86_page_table.omg','pub machine x86_decompose_bootstrap_virtual_address')
PHYS=ref(CORE+'x86_page_table.omg','pub machine x86_validate_bootstrap_physical_frame_geometry')
APIC=ref(FACT+'local_apic.omg','pub const IA32_APIC_BASE_MSR')
PORT=ref(CORE+'pic_8259.omg','reaches PortIo')
MSR=ref(CORE+'x2apic_timer.omg','reaches MachineControl')
EXTENT=ref(CORE+'extent.omg','ExtentRootProvider')
PROFILE=ref(CORE+'x86_interrupt_profile.omg','pub machine')
ROOT=ref(CORE+'legacy_timer_root.omg','InterruptAcknowledgement')
# Every source file is explicit. Related targets establish collisions/authority
# precedents only, never wholesale equivalence of an upstream module.
MODULES={
'src/lib.rs':([F,A,R],'PrivilegeLevel and numeric conversion missing; root exports/sealed Rust trait are packaging only.',[]),
'src/addr.rs':([F,A,E,P,T],'48-bit canonical virtual and52-bit physical values, alignment, checked stepping and arithmetic. Existing core has bootstrap-only validation/decomposition; extract/reuse numeric behavior without duplicating a core policy or turning numeric pointers into access.',[ADDR,PHYS]),
'src/instructions/mod.rs':([I,R],'hlt/nop/read_rip/debug breakpoint are instruction catalog or target-debug operations. Module exports are packaging; no ambient instruction wrapper API.',[MSR]),
'src/instructions/interrupts.rs':([I,P,R],'IF observation, cli/sti, interrupt-disabled closures, sti+hlt, int3/software interrupt require admitted execution and exact save/restore lifecycle, not arbitrary safe closures.',[ROOT,PROFILE]),
'src/instructions/port.rs':([I,R],'in/out8/16/32 require PortIo. Rust Port access marker/ownership wrappers and clone do not establish Cathedral authority; retain numeric port facts separately.',[PORT]),
'src/instructions/random.rs':([I,T],'CPUID feature discovery and RDRAND are effectful instruction boundaries; returned random values are not pure deterministic helpers. Feature-dependent unit test is hardware-only.',[]),
'src/instructions/segmentation.rs':([I,P],'Segment get/set, far CS transfer, FS/GS base and swapgs require current CPU/segment state and lifetime ownership; selector encoding is in registers/segmentation.',[]),
'src/instructions/smap.rs':([I,P,R],'CPUID/CR4/AC observations and stac/clac guards require checked nested restoration; freely constructed Smap token is not an authority. Boolean CPU feature observations may feed pure plans.',[]),
'src/instructions/tables.rs':([I,P],'lgdt/lidt/sgdt/sidt/ltr need admitted descriptor backing, stable lifetime and CPU installation; DescriptorTablePointer bytes are separate pure representation.',[IDT,IST]),
'src/instructions/tlb.rs':([F,A,I,P],'PCID bounds, INVPCID descriptor/command and broadcast batch arithmetic are pure work; CPUID, invlpg/invpcid/invlpgb/tlbsync and CR3 reload are instruction boundaries with mapping-era/processor scope.',[WALK]),
'src/registers/mod.rs':([R],'Module/reexport packaging only; per-register facts and instruction methods classified below.',[]),
'src/registers/control.rs':([F,A,I,P],'CR0/CR3/CR4 flags and CR8 priority encodings missing; read/write/update including PCID/no-flush paths need MachineControl and reserved-bit/CPU-feature/mapping policy.',[ADDR,MSR]),
'src/registers/debug.rs':([F,A,I],'DR6/DR7 flags, register index, breakpoint size/condition codecs and Dr7Value manipulation are pure missing work; debug register read/write/update remain boundaries.',[]),
'src/registers/model_specific.rs':([F,A,E,I,P],'MSR identifiers, EFER/CET flags, STAR selector checks/encoding are pure missing work. IA32_APIC_BASE identifier/bits already in facts; all register I/O remains admitted instruction work.',[APIC,MSR]),
'src/registers/mxcsr.rs':([F,A,I,T],'MXCSR flags/default bits are pure missing facts; SSE control-state read/write/update and related tests are CPU state operations.',[]),
'src/registers/rflags.rs':([F,I,T],'RFLAGS bits are pure facts. pushfq/popfq state observation/mutation and update closures require instruction contracts; read test is hardware-dependent.',[ROOT]),
'src/registers/segmentation.rs':([F,A,I],'SegmentSelector index/RPL/NULL encoding and privilege conversion are missing pure work. Segment/Segment64 methods require admitted CPU operations; zero-sized CS/DS/etc are Rust names, not capabilities.',[]),
'src/registers/xcontrol.rs':([F,I,P],'XCR0 flags are pure data; xgetbv/xsetbv/update requires OSXSAVE/feature consistency and live extended-state ownership.',[]),
'src/structures/mod.rs':([F,R,T],'Packed DescriptorTablePointer(limit,base) missing; base is inert address. Submodule exports packaging. Size test is a target representation vector candidate.',[]),
'src/structures/gdt.rs':([F,A,I,P,T],'GDT words, descriptor flags/default encodings, selector/DPL and TSS descriptor composition are pure missing work; TSS-pointer lifetime/bitmap memory observation and lgdt are boundaries; selecting entries remains OS policy.',[IST]),
'src/structures/idt.rs':([E,F,A,I,P,R,T],'Reuse X86IdtGate and exception vectors, extend pure options/error-code codecs and detached stack-frame facts. Rust x86-interrupt handler types/macros, live frame volatile mutation and iretq do not replace admitted roots.',[IDT,EXC,IST,PROFILE,ROOT]),
'src/structures/mem_encrypt.rs':([F,A,P,R],'EncryptedBit/SharedBit configuration and mask transforms can become explicit pure inputs. Reject global mutable address-mask configuration as library authority; CPU profile and re-admission of existing mappings belong to Cathedral.',[PTE,WALK]),
'src/structures/paging/mod.rs':([R],'Reexport-only module; preserve visibility map without a parallel paging authority package.',[PTE,WALK]),
'src/structures/paging/frame.rs':([F,A,E,T],'PhysFrame size/PFN/range arithmetic are inert numeric work with no allocation claim. Existing bootstrap4KiB/52-bit PFN validator covers a narrow subset; larger frames/range tests remain missing.',[PHYS]),
'src/structures/paging/frame_alloc.rs':([P,R],'Unique unused frame ownership and safe deallocation belong to qualified Extent allocator contracts, not a copied unsafe trait returning freely forgeable numeric frames.',[EXTENT]),
'src/structures/paging/page.rs':([F,A,E,T],'4KiB/2MiB/1GiB page facts, index composition and canonical-gap range stepping are missing general pure work; existing core four-level4KiB decomposition remains policy.',[ADDR]),
'src/structures/paging/page_table.rs':([E,F,A,P,T],'Reuse complete14-field X86PageTableEntry/8byte layout and512-entry candidate. Flags/word codecs, PageTableIndex/PageOffset/level helpers and detached table operations are missing. Bit7 leaf PAT vs upper huge-page meaning and bits59..62 key interpretation are role-dependent.',[PTE,WALK]),
'src/structures/paging/mapper/mod.rs':([F,A,P,I,R],'Numeric TranslateResult/errors and detached walk recipes can port. Live Mapper/CleanUp and physical backing depend on Extent/placement/TLB policy. Reject must_use-only flush debts and unrestricted ignore() as acknowledgement authority.',[WALK,EXTENT]),
'src/structures/paging/mapper/mapped_page_table.rs':([A,P,R],'Map/unmap/flag/translation/cleanup algorithms may become detached plans over admitted table views. Reject arbitrary PhysFrame-to-*mut PageTable conversion as access; live walker must prove backing, alias/lifetime and hierarchy roles.',[WALK,EXTENT]),
'src/structures/paging/mapper/offset_page_table.rs':([A,P,R],'Direct-map offset arithmetic is pure candidate work. Choosing global physical offset and casting resulting addresses into references is Cathedral mapping policy, not a numeric conversion grant.',[WALK,EXTENT]),
'src/structures/paging/mapper/recursive_page_table.rs':([A,P,I,R],'Recursive-coordinate arithmetic can port independently. Active-CR3 checks, recursive mapping choice and PRESENT|WRITABLE insertion are policy; raw table pointer traversal needs authority. Upstream warns index511 pointer-end unsoundness.',[WALK,EXTENT]),
'src/structures/port.rs':([I,R],'PortRead/PortWrite unsafe primitive traits are subsumed by checked instruction contracts and PortIo service; no trait copied as an authority grant.',[PORT]),
'src/structures/tss.rs':([F,A,P,T],'104byte TSS fixed record, default and I/O bitmap validation errors missing. Existing stack/IST role assignments must be reused; live backing/bitmap lifetime/loading remain authority policy.',[IST,PROFILE]),
}
TESTING={
'gdt.rs':'Test-specific lazy static GDT/TSS/stack and segment loading: retain intent, replace allocation/root lifecycle with Cathedral harness.',
'lib.rs':'bootloader0.9 _start, panic handler, serial runner and QEMU debug exit are Rust test harness, not target library API.',
'serial.rs':'spin/lazy_static UART helper is test harness with PortIo; reuse Cathedral serial facts/provider instead of duplicate ambient port.',
 'tests.rs':'Example 1==1 test validates Rust harness only, no hardware algorithm to translate.',
}
for name,reason in TESTING.items():MODULES['testing/src/'+name]=([T,P,R],reason,[])
for name,reason in {
'basic_boot.rs':'Rust boot entry/println smoke intent; Cathedral production boot smoke owns actual entry.',
'double_fault_stack_overflow.rs':'Deliberate recursive stack overflow and alternate double-fault stack test needs admitted fault roots/IST backing; cannot run as a pure host test.',
'interrupt_handling.rs':'Breakpoint/software-interrupt handlers and atomics test actual x86 interrupt ABI; retain scenarios against Cathedral roots, never arbitrary function-pointer installation.',
'port_read_write.rs':'CRT index/data hardware roundtrip is PortIo test, not pure fact validation; preserve exact restoration/authorization scenario.',
}.items():MODULES['testing/tests/'+name]=([T,I,P],reason,[ROOT] if 'fault' in name or 'interrupt' in name else [PORT])
