#!/usr/bin/env python3
"""All eight instruction modules audited; TLB numeric slice individually transcribed."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];checkout=ROOT/'reference_code/rust-osdev/x86_64'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
files=sorted(str(p.relative_to(checkout)) for p in (checkout/'src/instructions').glob('*.rs'))
m=shared.snapshot(checkout,'cc35c876d3badb57df54a66e22f7768a52be95f2',files,'https://github.com/rust-osdev/x86_64')
facts='source/drivers/facts/x86_tlb_operands.omg';helpers='source/libraries/x86_64/tlb_operands.omg'
methods={33:(helpers,'pub data InvPcidCommand'),56:(facts,'pub data InvpcidDescriptor'),64:(facts,'pub data Pcid '),69:(helpers,'pub machine pcid_new'),78:(helpers,'pub machine pcid_value'),87:(helpers,'pub data PcidResult'),101:(helpers,'pub machine invpcid_prepare'),146:(facts,'pub data InvlpgbLimits'),158:(helpers,'pub machine limits_from_observations'),186:(facts,'count_max: u16'),192:(facts,'nested: bool'),198:(facts,'nasid: u32'),203:(helpers,'pub machine broadcast_default'),228:(helpers,'pub data BroadcastRequest'),249:(helpers,'pub machine with_pages'),269:(helpers,'pub machine with_pcid'),280:(helpers,'state asid_valid'),293:(helpers,'pub machine with_global'),299:(helpers,'pub machine with_final_only'),305:(helpers,'pub machine with_nested'),316:(helpers,'pub machine pinned_range_step'),373:(helpers,'case AsidOutOfRange'),375:(helpers,'case AsidOutOfRange'),377:(helpers,'case AsidOutOfRange'),392:(helpers,'pub machine broadcast_prepare')}
notes={
'interrupts.rs':'RFLAGS predicate facts already exist; IF mutation, software interrupt entry, callback restoration and enable+halt are instruction/provider lifecycle work. No callback execution plan is supplied by numeric data.',
'mod.rs':'Module scaffolding or individual instruction leaves (halt/no-op/debug/current RIP); no independent numeric operand algorithm remains here.',
'port.rs':'Port numbers remain u16 and existing PIC PortIo operations remain canonical. Rust read/write marker wrappers, Copy/Clone and construction are deliberately not port-access authority. Generic width-specific leaves require admitted PortIo integration; formatting/equality are ordinary omitted API engineering.',
'random.rs':'CPUID bit30 predicate is an ordinary extractable observation check; obtaining random bytes and carrying provider success require RDRAND boundary work. No standalone CPUID/RNG provider introduced in this TLB slice.',
'segmentation.rs':'Existing SegmentSelector, FS_BASE_MSR and GS_BASE_MSR facts are canonical. Segment register mutation, base reads/writes, CS far return and swapgs are instruction/entry authority boundaries.',
'smap.rs':'Existing RFLAGS AC bit and CR4 SMAP flag are canonical. CPUID bit20 interpretation is ordinary extractable observation engineering; STAC/CLAC and callback restoration require a checked provider/guard lifecycle.',
 'tables.rs':'Existing DescriptorTablePointer and selector values/codecs are canonical. LGDT/LIDT/SGDT/SIDT/LTR need instruction and admitted table/backing lifetimes.',
}
for path,file in m['files'].items():
 is_tlb=path.endswith('/tlb.rs')
 file.update(disposition='translated' if is_tlb else 'omitted',reason='Pure operand extraction; hardware tails explicitly excluded.' if is_tlb else notes[Path(path).name])
 if is_tlb:file['targets']=[{'path':helpers,'anchor':'module tlb_operands;'}]
 for key,row in file['symbols'].items():
  line,name=key.split(':',1);target=methods.get(int(line)) if is_tlb else None
  if target:
   reason='Pure values/validation/composition only; no probe, instruction execution or invalidation settlement.'
   if int(line)==316:reason='Exact one-iteration pinned recipe. Architectural count interpretation is separately implemented and tested; no flush loop or CPU completion authority.'
   if int(line) in (269,280,305):reason='Detached selection mutation is independent of execution authority; prepare validates ASID/nested limits. CR4.PCIDE/EFER.SVME/provider provenance remain separate obligations.'
   row.update(disposition='translated',reason=reason,targets=[{'path':target[0],'anchor':target[1]}])
  else:
   reason='TLB instruction execution, deprecated misspelled alias or Rust formatting deliberately omitted.' if is_tlb else notes[Path(path).name]
   row.update(disposition='omitted',reason=reason)
m['private_and_implicit_mappings']=[{'source':'InvPcidCommand::{Address,Single,All,AllExceptGlobal}','target':helpers+'::InvPcidCommand','meaning':'Semantic cases retained, canonical address checked before producing operand.'},{'source':'InvpcidDescriptor::{pcid,address}','target':facts+'::InvpcidDescriptor','meaning':'Exact raw128-bit fixed record, separate16-byte codec and requested Layout policy.'},{'source':'Invlpgb private limits and InvlpgbFlushBuilder optional selections/booleans','target':facts+'::InvlpgbLimits and '+helpers+'::BroadcastRequest','meaning':'Explicit observed numeric inputs, semantic option cases; no borrowed live CPU object.'}]
m['architectural_discrepancy']={'source':'AMD24594 r3.36 p388','rule':'ECX low16 is additional-page count; addressed total=encoded+1','pinned':'count capped by encoded maximum; advance=max(count,1)','separate_recipe':'architectural_range_step caps addressed total by maximum+1, encodes total-1 and advances total','meaning':'No silent change to pinned translation and no hardware execution claim.'}
p=ROOT/'source/libraries/x86_64/tlb-operands-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('inventory drift')
else:p.write_text(text)
print(shared.check(m,checkout))
