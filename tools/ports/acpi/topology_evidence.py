#!/usr/bin/env python3
"""Exact source-map evidence for inert CPU/controller/timer/PCI extraction."""
import argparse,json
from fixed_model import ROOT,UP,PIN
from header_evidence import api,translated,omitted
DEST=ROOT/'source/libraries/acpi'

def evidence():
 inv=api.snapshot(UP,PIN,['src/platform/mod.rs','src/platform/interrupt.rs','src/platform/pci.rs'],'https://github.com/rust-osdev/acpi')
 inv['scope']='ACPI003 ordered inert descriptions and pure queries. Replaces allocator-owned aggregate topology with bounded per-record extraction. NUMA and live platform operations excluded; complete inventory retains them pending.'
 for path,file in inv['files'].items():
  translated(file,'topology.PORT.md','## Source map','Partial API translation; every anchor classified below.')
  for key,row in file['symbols'].items():
   line,name=key.split(':',1);line=int(line)
   if path.endswith('/pci.rs'):
    if name=='physical_address':translated(row,'pci_regions.omg','machine physical_address','Checked bus0-relative ECAM numeric address; explicit field bounds/overflow/ambiguous-range errors. No pointer or MMIO authority.')
    elif name=='regions':translated(row,'pci_regions.omg','regions: &[PciConfigRegion; 253]','Explicit checked-count fixed input replaces allocator vector; regions obtained from checked pci_region.')
    else:omitted(row,'Allocator-owned mapped-table constructor and container identity replaced by explicit bounded pci_region/find_address inputs; no AcpiTables handler, global allocator or mapped-memory authority is introduced.')
   elif path.endswith('/interrupt.rs'):
    if name in ['IoApic','NmiLine','LocalInterruptLine','NmiProcessor','InterruptSourceOverride','NmiSource']:
     translated(row,'topology.omg','data '+name)
    elif 253<=line<=299:translated(row,'topology.omg',name+':','Owned inert field; local NMI additionally retains polarity/trigger facts the pin drops.')
    elif name=='from_apic_model_in':translated(row,'topology.omg','machine topology_fact','Pure per-record extraction rewritten as ordered bounded stream. Allocator aggregate, first-CPU live-state inference, and model-selection side effects absent; GIC and unknown entries retained.')
    elif name=='local_apic_address':translated(row,'topology.omg','initial_local_apic_address','Initial header address plus separately ordered LocalApicAddress override events; not claimed final until consumer finishes stream.')
    elif name=='also_has_legacy_pics':translated(row,'topology.omg','also_has_legacy_pics:')
    elif name.startswith('crate::'):translated(row,'topology.omg','use madt::Polarity;')
    elif name in ['io_apics','local_apic_nmi_lines','interrupt_source_overrides','nmi_sources']:
     translated(row,'topology.omg','data TopologyFact','Ordered typed entries replace vectors; no entries silently lost. Collection/storage choice remains caller-owned.')
    else:omitted(row,'Whole mapped/allocator aggregate and model-selection API replaced by explicit topology_header/topology_next stream. No full AcpiTables constructor or global InterruptModel identity is claimed.')
   else:
    if name in ['Processor','processor_uid','local_apic_id'] and line>=214:translated(row,'topology.omg','data Processor'if name=='Processor'else name+':')
    elif name in ['state','is_ap'] and line>=214:translated(row,'topology.omg','enabled:'if name=='state'else'boot_identity_known:','Firmware enabled/online-capable flags and explicit observed boot identity replace inferred runtime state or first-entry BSP assumptions.')
    elif name=='PmTimer':translated(row,'timers.omg','data PmTimer')
    elif name in ['base','supports_32bit']:translated(row,'timers.omg',name+':')
    elif name=='new'and line==254:translated(row,'timers.omg','machine pm_timer','Pure PM timer description additionally honors primary hardware-reduced rule.')
    elif name=='numa':omitted(row,'NUMA/SRAT/SLIT branch not included in queue003 CPU/controller/timer/PCI slice; whole inventory remains pending.')
    else:omitted(row,'Live platform handler/mapped constructors, event/register operations, ACPI mode and AP wake effects, Rust allocator/module adapters or aggregate runtime-state API remain outside inert topology extraction. No behavior silently substituted with a successful stub.')
 return inv

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');args=p.parse_args();path=DEST/'topology-inventory.json';text=json.dumps(evidence(),indent=2,sort_keys=True)+'\n'
 if args.check:
  if not path.exists()or path.read_text()!=text:raise SystemExit('Topology inventory differs')
 else:path.write_text(text)
 print('Pinned topology source map '+('verified'if args.check else'generated'))
if __name__=='__main__':main()
