#!/usr/bin/env python3
"""Original inert topology expectations, including pin deviations."""
import argparse,json,re
from fixed_model import HERE,ROOT
from header_fixtures import sdt
IMPORTS={'topology':['TopologyHeader','topology_header','ObservedBootApic','Processor','processor','TopologyStep','TopologyFact','topology_fact','topology_next','NmiProcessor','LocalInterruptLine'],
 'madt':['MadtEntry','Polarity','TriggerMode'],
 'fixed_types':['LocalApicEntry','LocalX2ApicEntry','IoApicEntry','InterruptSourceOverrideEntry','NmiSourceEntry','LocalApicNmiEntry','X2ApicNmiEntry','LocalApicAddressOverrideEntry','MultiprocessorWakeupEntry','GiccEntry','GicdEntry','GicMsiFrameEntry','GicRedistributorEntry','GicInterruptTranslationServiceEntry','EntryHeader','Fadt'],
 'timers':['pm_timer','PmTimerResult'],
 'gas':['OptionalRawGas','RawGenericAddress'],
 'pci_regions':['PciConfigRegion','PciRegionResult','PciAddressResult','physical_address','find_address','pci_region']}


def cases():
 rows=[]
 def add(name,body,checks,data=b'',helper=None):rows.append(dict(name=name,body=body,checks=checks,bytes=bytes(data).hex(),helper=helper))
 for flags in [0,1,2,3,0xffffffff]:
  for known,boot in [(False,42),(True,42),(True,43)]:
   add(f'cpu_{flags}_{known}_{boot}',[f'let value: Processor = processor(4294967295, 42, {flags}, true, ObservedBootApic {{ known: {str(known).lower()}, id: {boot} }});'],['value.processor_uid == 4294967295','value.local_apic_id == 42','value.x2apic',f'value.enabled == {str(bool(flags&1)).lower()}',f'value.online_capable == {str(bool(flags&2)).lower()}',f'value.boot_identity_known == {str(known).lower()}',f'value.is_boot_processor == {str(known and boot==42).lower()}'])
 variants=[
 ('LocalApic','local_apic_entry','LocalApicEntry { processor_id: 3, apic_id: 42, flags: 3 }','Cpu','cpu','cpu.processor_uid == 3 && cpu.local_apic_id == 42 && !cpu.x2apic && cpu.enabled && cpu.online_capable && cpu.is_boot_processor'),
 ('LocalX2Apic','local_x2_apic_entry','LocalX2ApicEntry { processor_uid: 4294967295, x2apic_id: 42, flags: 1 }','Cpu','cpu','cpu.processor_uid == 4294967295 && cpu.x2apic && cpu.enabled && !cpu.online_capable && cpu.is_boot_processor'),
 ('IoApic','io_apic_entry','IoApicEntry { io_apic_id: 255, io_apic_address: 4294967295, global_system_interrupt_base: 0x12345678 }','IoController','controller','controller.id == 255 && controller.address == 4294967295 && controller.global_system_interrupt_base == 0x12345678'),
 ('InterruptSourceOverride','interrupt_source_override_entry','InterruptSourceOverrideEntry { bus: 0, irq: 255, global_system_interrupt: 4294967295, flags: 15 }','Override','interrupt_override','interrupt_override.isa_source == 255 && interrupt_override.global_system_interrupt == 4294967295'),
 ('NmiSource','nmi_source_entry','NmiSourceEntry { global_system_interrupt: 4294967295, flags: 15 }','GlobalNmi','global_nmi','global_nmi.global_system_interrupt == 4294967295'),
 ('LocalApicAddressOverride','local_apic_address_override_entry','LocalApicAddressOverrideEntry { local_apic_address: 18446744073709551615 }','LocalApicAddress','local_apic_address','local_apic_address == 18446744073709551615'),
 ('MultiprocessorWakeup','multiprocessor_wakeup_entry','MultiprocessorWakeupEntry { mailbox_address: 18446744073709551615, mailbox_version: 65535 }','WakeMailbox','mailbox_address, mailbox_version','mailbox_address == 18446744073709551615 && mailbox_version == 65535'),
 ('Gicc','gicc_entry','GiccEntry { processor_uid: 42, mpidr: 18446744073709551615 }','GicCpu','gic_cpu','gic_cpu.processor_uid == 42 && gic_cpu.mpidr == 18446744073709551615'),
 ('Gicd','gicd_entry','GicdEntry { gic_id: 42, physical_base_address: 18446744073709551615 }','GicDistributor','gic_distributor','gic_distributor.gic_id == 42 && gic_distributor.physical_base_address == 18446744073709551615'),
 ('GicMsiFrame','gic_msi_frame_entry','GicMsiFrameEntry { frame_id: 42, physical_base_address: 18446744073709551615 }','GicMsi','gic_msi','gic_msi.frame_id == 42 && gic_msi.physical_base_address == 18446744073709551615'),
 ('GicRedistributor','gic_redistributor_entry','GicRedistributorEntry { discovery_range_base_address: 18446744073709551615, discovery_range_length: 4294967295 }','GicRedistributor','gic_redistributor','gic_redistributor.discovery_range_base_address == 18446744073709551615 && gic_redistributor.discovery_range_length == 4294967295'),
 ('GicInterruptTranslationService','gic_interrupt_translation_service_entry','GicInterruptTranslationServiceEntry { id: 42, physical_base_address: 18446744073709551615 }','GicTranslation','gic_translation','gic_translation.id == 42 && gic_translation.physical_base_address == 18446744073709551615')]
 for variant,field,value,result,payload,expect in variants:
  name='fact_'+variant
  helper=f'machine check_{name}(fact: TopologyFact) -> bool {{ transition fact {{ TopologyFact::{result} {{ {payload} }} -> ({expect}) _ -> (false) }} }}'
  add(name,[f'let result: TopologyStep = topology_fact(MadtEntry::{variant} {{ decoded_{field}: {value} }}, ObservedBootApic {{ known: true, id: 42 }});',f'let fields: bool = check_{name}(result.fact);'],['result.error == 0','fields'],helper=helper)
 for flags,error in [(2,15),(8,16),(15,0)]:
  add('iso_flags_'+str(flags),[f'let result: TopologyStep = topology_fact(MadtEntry::InterruptSourceOverride {{ decoded_interrupt_source_override_entry: InterruptSourceOverrideEntry {{ flags: {flags} }} }}, ObservedBootApic {{}});'],[f'result.error == {error}'])
 add('iso_bus',['let result: TopologyStep = topology_fact(MadtEntry::InterruptSourceOverride { decoded_interrupt_source_override_entry: InterruptSourceOverrideEntry { bus: 1 } }, ObservedBootApic {});'],['result.error == 32'])
 for x2 in [False,True]:
  for uid in [42,4294967295 if x2 else 255]:
   for line in [0,1,2]:
    kind='X2ApicNmi'if x2 else'LocalApicNmi';field='x2_apic_nmi_entry'if x2 else'local_apic_nmi_entry';u='processor_uid'if x2 else'processor_id';name=f'nmi_{x2}_{uid}_{line}'
    target='NmiProcessor::All' if uid!=42 else 'NmiProcessor::ProcessorUid { uid }'
    uid_check='' if uid!=42 else 'uid == 42 && '
    helper=f'machine check_{name}(fact: TopologyFact) -> bool {{ transition fact {{ TopologyFact::LocalNmi {{ local_nmi }} -> inspect(local_nmi.processor, local_nmi.line) _ -> (false) }} state inspect(processor: NmiProcessor, line: LocalInterruptLine) -> bool {{ let result: bool = check_target_{name}(processor, line); result }} }}\nmachine check_target_{name}(processor: NmiProcessor, line: LocalInterruptLine) -> bool {{ let correct_line: bool = line_{name}(line); transition processor {{ '+target+' -> ('+uid_check+'correct_line) _ -> (false) } }\nmachine line_'+name+'(line: LocalInterruptLine) -> bool { transition line { LocalInterruptLine::'+('Lint0'if line==0 else'Lint1')+' -> (true) _ -> (false) } }'
    add(name,[f'let result: TopologyStep = topology_fact(MadtEntry::{kind} {{ decoded_{field}: {kind}Entry {{ {u}: {uid}, nmi_line: {line}, flags: 15 }} }}, ObservedBootApic {{}});']+([f'let fields: bool = check_{name}(result.fact);']if line<2 else []),[f'result.error == {0 if line<2 else 33}']+(['fields']if line<2 else []),helper=helper if line<2 else None)
 # Byte pipeline preserves source order; caller evidence identifies second CPU as BSP.
 data=sdt(b'APIC',bytes(8)+bytes([0,8,3,41,1,0,0,0,0,8,4,42,1,0,0,0]))
 helper='machine pipeline_cpu(fact: TopologyFact, expected: u32, boot: bool) -> bool { transition fact { TopologyFact::Cpu { cpu } -> (cpu.local_apic_id == expected && cpu.is_boot_processor == boot) _ -> (false) } }'
 add('ordered_cpu_pipeline',['let first: TopologyStep = topology_next(&input, 60, 44, ObservedBootApic { known: true, id: 42 });','let second: TopologyStep = topology_next(&input, 60, first.next_offset, ObservedBootApic { known: true, id: 42 });','let end: TopologyStep = topology_next(&input, 60, second.next_offset, ObservedBootApic {});','let a: bool = pipeline_cpu(first.fact, 41, false);','let b: bool = pipeline_cpu(second.fact, 42, true);'],['first.error == 0','second.error == 0','first.source_offset == 44','second.source_offset == 52','end.done','a','b'],data,helper)
 add('topology_malformed',['let result: TopologyStep = topology_next(&input, 46, 44, ObservedBootApic {});'],['result.error == 14'],sdt(b'APIC',bytes(8)+bytes([128,0])))
 helper='machine unknown_kept(fact: TopologyFact) -> bool { transition fact { TopologyFact::Other { other_entry } -> inspect(other_entry) _ -> (false) } state inspect(entry: MadtEntry) -> bool { let result: bool = raw_unknown(entry); result } }\nmachine raw_unknown(entry: MadtEntry) -> bool { transition entry { MadtEntry::Unknown { header } -> (header.entry_type == 128 && header.length == 2) _ -> (false) } }'
 add('topology_unknown',['let result: TopologyStep = topology_next(&input, 46, 44, ObservedBootApic {});','let kept: bool = unknown_kept(result.fact);'],['result.error == 0','result.next_offset == 46','kept'],sdt(b'APIC',bytes(8)+bytes([128,2])),helper)
 for flags in [0,256,1048576,1048832]:
  add('pm_timer_'+str(flags),[f'let result: PmTimerResult = pm_timer(Fadt {{ flags: {flags}, pm_timer_length: 4, pm_timer_block: 4660 }});'],['result.error == 0',f'result.present == {str(not flags&1048576).lower()}',f'result.hardware_reduced == {str(bool(flags&1048576)).lower()}']+(['result.value.base.address == 4660',f'result.value.counter_bits == {32 if flags&256 else 24}']if not flags&1048576 else []))
 add('pm_absent',['let result: PmTimerResult = pm_timer(Fadt {});'],['!result.present','result.error == 0'])
 add('pm_reduced_invalid_ignored',['let result: PmTimerResult = pm_timer(Fadt { flags: 1048576, pm_timer_length: 4, x_pm_timer_block: OptionalRawGas { present: true, value: RawGenericAddress { address_space: 12, address: 1234 } } });'],['!result.present','result.error == 0','result.hardware_reduced'])
 add('pm_invalid_gas',['let result: PmTimerResult = pm_timer(Fadt { pm_timer_length: 4, x_pm_timer_block: OptionalRawGas { present: true, value: RawGenericAddress { address_space: 12, address: 1234 } } });'],['result.error == 11'])
 for bus,device,function in [(0,0,0),(128,0,0),(255,31,7),(1,32,0),(1,0,8)]:
  error=34 if device>31 else 35 if function>7 else 0
  add(f'ecam_{bus}_{device}_{function}',[f'let result: PciAddressResult = physical_address(PciConfigRegion {{ base_address: 2147483648, segment: 4660, first_bus: 0, last_bus: 255 }}, 4660, {bus}, {device}, {function});'],[f'result.error == {error}']+(['result.found',f'result.address == {0x80000000+(bus<<20)+(device<<15)+(function<<12)}']if not error else []))
 add('ecam_nonzero_first',['let result: PciAddressResult = physical_address(PciConfigRegion { base_address: 2147483648, segment: 4660, first_bus: 128, last_bus: 255 }, 4660, 128, 0, 0);'],['result.error == 0','result.address == 2281701376','result.found'])
 for base,bus,error in [(0xffffffffffffffff,0,0),(0xffffffffffffffff,1,36),(0xffffffffffefffff,1,0)]:
  add('ecam_overflow_'+str(base)+'_'+str(bus),[f'let result: PciAddressResult = physical_address(PciConfigRegion {{ base_address: {base}, last_bus: 255 }}, 0, {bus}, 0, 0);'],[f'result.error == {error}']+([f'result.address == {base+(bus<<20)}']if not error else []))
 for segment,bus in [(1,128),(0,127),(0,201)]:
  add(f'ecam_no_region_{segment}_{bus}',[f'let result: PciAddressResult = physical_address(PciConfigRegion {{ first_bus: 128, last_bus: 200 }}, {segment}, {bus}, 0, 0);'],['result.error == 0','!result.found'])
 for count,error in [(0,0),(1,0),(2,37),(254,1)]:
  add('regions_count_'+str(count),['let mut regions: [PciConfigRegion; 253];','regions[0] = PciConfigRegion { base_address: 2147483648, last_bus: 255 };','regions[1] = PciConfigRegion { base_address: 4294967296, first_bus: 128, last_bus: 255 };',f'let result: PciAddressResult = find_address(&regions, {count}, 0, 128, 0, 0);'],[f'result.error == {error}']+(['result.address == 2281701376','result.found']if count==1 else ['!result.found']if count==0 else []))
 add('regions_disjoint',['let mut regions: [PciConfigRegion; 253];','regions[0] = PciConfigRegion { base_address: 2147483648, last_bus: 127 };','regions[1] = PciConfigRegion { base_address: 4294967296, first_bus: 128, last_bus: 255 };','let result: PciAddressResult = find_address(&regions, 2, 0, 128, 0, 0);'],['result.error == 0','result.address == 4429185024','result.found'])
 payload=bytes(8)+0x80000000.to_bytes(8,'little')+bytes([0x34,0x12,128,255,0,0,0,0])
 add('region_bytes',['let result: PciRegionResult = pci_region(&input, 60, false, 0);'],['result.error == 0','result.value.base_address == 2147483648','result.value.segment == 4660','result.value.first_bus == 128','result.value.last_bus == 255'],sdt(b'MCFG',payload))
 for flags in [0,1,2,3]:
  data=sdt(b'APIC',0xfee00000.to_bytes(4,'little')+flags.to_bytes(4,'little'),6)
  add('topology_header_'+str(flags),['let result: TopologyHeader = topology_header(&input, 44);'],['result.error == 0','result.initial_local_apic_address == 4276092928','result.revision == 6',f'result.also_has_legacy_pics == {str(bool(flags&1)).lower()}'],data)
 for device,function,error in [(32,0,34),(0,8,35)]:
  add(f'empty_region_invalid_{device}_{function}',['let mut regions: [PciConfigRegion; 253];',f'let result: PciAddressResult = find_address(&regions, 0, 0, 0, {device}, {function});'],[f'result.error == {error}'])
 return rows


def render(rows,evaluate=True):
 out=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Original inert topology semantic fixtures.']
 for module,names in IMPORTS.items():out +=[f'use acpi::{module}::{name};'for name in names]
 for row in rows:
  if row['helper']:out.append(row['helper'])
  out += [f'machine test_{row["name"]}() -> bool {{','    let mut input: [u8; 4096];']
  out += [f'    input[{i}] = {b};'for i,b in enumerate(bytes.fromhex(row['bytes']))if b]
  out += ['    '+line for line in row['body']]+['    '+' &&\n    '.join(row['checks']),'}']
 if evaluate:
  out += ['machine test_result() -> i32 { transition '+' && '.join('test_'+r['name']+'()'for r in rows)+' { true -> (0) _ -> (1) } }','const TEST_RESULT: i32 = test_result();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(TEST_RESULT); }']
 else:out +=['data Main {}','machine Main::main(&mut self) {}']
 body='\n'.join(line for line in out if not line.startswith('use '))
 return '\n'.join(line for line in out if not line.startswith('use ')or re.search(r'\b'+line.rsplit('::',1)[1].rstrip(';')+r'\b',body))+'\n'


def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();rows=cases()
 outputs={HERE/'topology-cases.json':json.dumps({'format':'cathedral-acpi-topology-cases-v1','provenance':'Original synthetic initialized bytes and pure descriptions; no firmware capture.','cases':rows},indent=2)+'\n',HERE/'topology_main.omg':render(rows,False)}
 for path,text in outputs.items():
  if args.check:
   if not path.exists()or path.read_text()!=text:raise SystemExit('Fixture differs: '+str(path))
  else:path.write_text(text)
 print(len(rows),'topology cases '+('verified'if args.check else'generated'))

if __name__=='__main__':main()
