#!/usr/bin/env python3
"""Bounded source mapping: pure descriptor values/plans; no full IDT/GDT authority."""
from pathlib import Path
import importlib.util,json,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
schema=json.loads((HERE/'schema.json').read_text());checkout=ROOT/'reference_code/rust-osdev/x86_64'
m=shared.snapshot(checkout,schema['revision'],schema['files'],'https://github.com/rust-osdev/x86_64')
facts='source/drivers/facts/x86_descriptors.omg';helpers='source/libraries/x86_64/descriptors.omg';tests='tools/ports/x86_64-descriptors/src/tests.rs';measure='tools/ports/x86_64-descriptors/measure.py'
constants={c['name'] for c in schema['constants']}
methods={
 'src/structures/mod.rs':{19:(facts,'pub data DescriptorTablePointer'),21:(facts,'limit: u16'),23:(facts,'base: u64'),32:(measure,'DescriptorTablePointer')},
 'src/structures/tss.rs':{14:(facts,'pub data TaskStateSegment'),18:(facts,'privilege_stack_table:'),22:(facts,'interrupt_stack_table:'),27:(facts,'iomap_base:'),38:(helpers,'pub machine tss_new'),53:(helpers,'pub machine tss_new'),60:(helpers,'pub data IoMapResult'),122:(measure,'TaskStateSegment')},
 'src/structures/gdt.rs':{3:('source/drivers/facts/x86_registers.omg','pub data SegmentSelector'),26:(facts,'pub data GdtEntry'),30:(facts,'pub data GdtEntry'),38:(facts,'raw: u64'),130:(helpers,'pub machine gdt_capacity_valid'),137:(helpers,'first_word == 0'),161:(helpers,'pub machine gdt_raw_metadata_valid'),197:(helpers,'pub machine gdt_append_plan'),250:(helpers,'pub machine gdt_append_plan'),258:(helpers,'pub machine gdt_limit'),281:(facts,'pub data Descriptor '),294:(facts,'pub data DescriptorFlags'),383:(helpers,'pub machine descriptor_dpl'),395:(helpers,'pub machine kernel_code_segment'),402:(helpers,'pub machine kernel_data_segment'),409:(helpers,'pub machine user_data_segment'),416:(helpers,'pub machine user_code_segment'),429:(helpers,'pub machine tss_descriptor('),441:(helpers,'pub machine tss_descriptor('),468:(helpers,'pub machine iomap_validate('),511:(helpers,'pub machine tss_descriptor('),543:(measure,'measurements'),576:(tests,'fn gdt_append_and_full_boundaries'),589:(tests,'fn gdt_append_and_full_boundaries'),596:(tests,'fn gdt_append_and_full_boundaries'),604:(tests,'fn gdt_append_and_full_boundaries'),612:(tests,'Descriptor::user_code_segment().dpl()')},
 'src/structures/idt.rs':{1218:(facts,'pub data SelectorErrorCode'),1224:(helpers,'pub machine selector_error_valid'),1233:(helpers,'pub machine selector_error_truncate'),1241:(helpers,'pub machine selector_error_external'),1246:(helpers,'pub machine selector_error_table'),1257:(helpers,'pub machine selector_error_index'),1262:(helpers,'pub machine selector_error_is_null'),1281:(facts,'pub const DESCRIPTOR_TABLE_GDT')},
}
for path,file in m['files'].items():
 file.update(disposition='translated',reason='Bounded pure descriptor slice; every other declaration explicitly omitted below. Source hashes include all branches/private fields.',targets=[{'path':facts,'anchor':'module x86_descriptors;'}])
 for key,row in file['symbols'].items():
  line,name=key.split(':',1);target=methods.get(path,{}).get(int(line))
  if path.endswith('/gdt.rs') and name in constants:target=(facts,'pub const DESCRIPTOR_'+name+':')
  if target:
   reason='Pure field/value/code or validation/encoding component translated; layout selection, native ABI and authority are separate evidence.'
   if path.endswith('/gdt.rs') and int(line) in {130,137,161,197,250}:reason='Pure geometry/metadata/append decision extracted. This target is a detached plan, not a claim to provide the complete generic mutable Rust GlobalDescriptorTable storage API.'
   if path.endswith('/gdt.rs') and int(line) in {429,441,468,511}:reason='Only numeric encoding and observed-byte validation translated. Pointer dereference, borrowed lifetime, persistent backing and live table installation deliberately excluded.'
   row.update(disposition='translated',reason=reason,targets=[{'path':target[0],'anchor':target[1]}])
  else:
   reason='Outside this bounded descriptor slice: live loading, full generic mutable GDT storage, concurrent/atomic entry observation, existing IDT/exception facts, other IDT APIs, Rust formatting/traits or scaffolding. Omission is not a compiler blocker.'
   if path.endswith('/tss.rs') and name=='was':reason='Shared lexical anchor inside a formatting string, not a declaration; complete original string retained by source hash.'
   row.update(disposition='omitted',reason=reason)
supp=json.loads((ROOT/'source/libraries/x86_64/lexical-supplement.json').read_text());m['supplemental_mappings']=[]
for path in schema['files']:
 for key,row in supp['files'].get(path,{}).get('symbols',{}).items():
  if any('::'+name+'::' in key for name in ('InvalidIoMap','Descriptor','DescriptorTable')):
   target=helpers if 'InvalidIoMap' in key else facts
   anchor='pub data IoMapResult' if 'InvalidIoMap' in key else 'pub data Descriptor ' if '::Descriptor::' in key else 'pub const DESCRIPTOR_TABLE_GDT'
   m['supplemental_mappings'].append({'source':path,'key':key,'anchor':row['anchor'],'target':{'path':target,'anchor':anchor},'reason':'Descriptor variants use matching semantic cases; error payloads remain explicit result fields. Local descriptor-table codes are explicit, not a native enum ABI.'})
m['layout_evidence']={'pointer':'Four 16-bit base fragments satisfy packed alignment; field access reaches known private-generated-field diagnostic. Not native measured.','tss':'Requested 104-byte/align4 geometry: ordinary u64 arrays at offsets4/36 reject native field-alignment validation. Complete explicit byte codecs are separate.','vectors':'32 actual pinned Rust host observations and UEFI-x64 assertions; no Omega native ABI claim.'}
p=ROOT/'source/drivers/facts/x86_descriptors-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('inventory changed')
else:p.write_text(text)
print(shared.check(m,checkout))
