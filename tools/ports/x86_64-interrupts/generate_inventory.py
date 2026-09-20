#!/usr/bin/env python3
"""Exact pinned IDT source disposition; numeric representations grant no authority."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
checkout=ROOT/'reference_code/rust-osdev/x86_64';path='src/structures/idt.rs'
m=shared.snapshot(checkout,'cc35c876d3badb57df54a66e22f7768a52be95f2',[path],'https://github.com/rust-osdev/x86_64')
facts='source/drivers/facts/x86_interrupts.omg';helpers='source/libraries/x86_64/interrupts.omg';codecs='source/libraries/x86_64/interrupt_bytes.omg';test='tools/ports/x86_64-interrupts/src/tests.rs';measure='tools/ports/x86_64-interrupts/measure.py'
methods={57:(facts,'pub data DetachedIdt'),453:(helpers,'pub machine idt_missing'),488:(helpers,'pub machine idt_reset'),532:(helpers,'pub machine idt_range'),554:(helpers,'pub machine idt_range'),563:(helpers,'pub machine idt_range'),571:(helpers,'pub machine idt_missing'),583:(helpers,'pub machine idt_index_kind'),613:(helpers,'pub machine idt_index_kind'),648:(helpers,'pub machine idt_range'),658:(helpers,'pub machine idt_range'),685:('source/drivers/facts/x86_idt_gate.omg','pub data X86IdtGate'),801:(helpers,'pub machine gate_missing'),828:(helpers,'pub machine gate_candidate'),845:(helpers,'pub machine gate_handler_canonical48'),915:(facts,'pub data EntryOptions'),936:(helpers,'pub machine options_minimal'),948:(helpers,'pub machine options_set_selector'),955:(helpers,'pub machine options_set_present'),960:(helpers,'pub machine options_present'),967:(helpers,'pub machine options_disable_interrupts'),975:(helpers,'pub machine options_set_privilege'),980:(helpers,'pub machine options_privilege'),998:(helpers,'pub machine options_set_stack_index'),1005:(helpers,'pub machine options_stack_index'),1018:(facts,'pub data RawInterruptStackFrame'),1023:(helpers,'pub machine frame_new'),1078:(facts,'pub data RawInterruptStackFrame'),1100:(helpers,'pub machine frame_new'),1171:(facts,'pub data PageFaultErrorCode'),1298:(helpers,'pub machine exception_vector_known'),1371:(helpers,'pub data ExceptionVectorCheck'),1385:(helpers,'pub machine exception_vector_check'),1643:(test,'#[test]'),1666:(measure,'measurements')}
schema=json.loads((HERE/'schema.json').read_text());flags={x['name'] for x in schema['flags']}
for key,row in m['files'][path]['symbols'].items():
 line,name=key.split(':',1);line=int(line);target=methods.get(line)
 if 65<=line<=421:target=(facts,'entries: [X86IdtGate; 256]')
 if name in flags:target=(facts,'pub const PAGE_FAULT_'+name+':')
 if 1084<=line<=1093:target=(facts,name+':')
 if 1300<=line<=1366:target=(helpers,'pub machine exception_vector_known')
 if 1218<=line<=1281 and line!=1268:
  old=json.loads((ROOT/'source/drivers/facts/x86_descriptors-inventory.json').read_text())['files'][path]['symbols'][key]
  target=(old['targets'][0]['path'],old['targets'][0]['anchor'])
 if target:
  reason='Pure numeric facts/behavior translated; raw bytes do not establish handler, live frame, CS, table lifetime or CPU authority.'
  if line in (57,488,554,563,583,613,648,658):reason='Detached array storage, numeric index/range classification and replacement/default primitives only; typed borrowed Rust indexing and live table API are not claimed.'
  if line==828:reason='Numeric candidate requires an explicit selector. Upstream current-CS read, unsafe validity obligations and handler authority deliberately excluded.'
  if 65<=line<=421:reason='Named upstream entry maps to its vector slot in canonical 256-entry array; actual public field offset verified in measure.py. Existing exception policy remains separate.'
  row.update(disposition='translated',reason=reason,targets=[{'path':target[0],'anchor':target[1]}])
 else:
  reason='Rust formatting/equality/scaffolding or integration-only test; no Omega algorithm claim and no compiler blocker.'
  if line in (495,511,521,870,880,882,893,1054,1060,1063,1131) or 720<=line<=796 or 1467<=line<=1639 or line>=1683:reason='Live instruction, handler function/calling convention, pointer lifetime, volatile saved-frame mutation, assembly return or handler macro integration deliberately excluded from detached numeric values.'
  row.update(disposition='omitted',reason=reason)
m['files'][path].update(disposition='translated',reason='Bounded inert interrupt representations and pure codecs; all anchors individually classified.',targets=[{'path':facts,'anchor':'module x86_interrupts;'}])
m['private_representation_mappings']=[
 {'source':'Entry::{pointer_low,pointer_middle,pointer_high}','target':codecs+'::encode_gate/decode_gate','meaning':'Explicit16/16/32-bit little-endian fragments of existing X86IdtGate.entry.'},
 {'source':'Entry::options / EntryOptions::{cs,bits}','target':facts+'::EntryOptions and '+helpers+'::gate_options/gate_from_options','meaning':'Selector renamed from cs; raw16-bit options preserved, constrained gate rejects reserved IST bits.'},
 {'source':'Entry::reserved','target':'source/drivers/facts/x86_idt_gate.omg::reserved','meaning':'Existing zero-domain field; parser rejects nonzero raw32-bit tail.'},
 {'source':'Entry::phantom','target':'deliberately omitted','meaning':'Rust handler-type marker supplies no valid Omega handler authority.'},
 {'source':'InterruptDescriptorTable private/reserved and interrupt array entries','target':facts+'::DetachedIdt.entries','meaning':'All256 numeric slots preserved; no competing named table field API or typed handler pointer.'},
 {'source':'InterruptStackFrameValue reserved fields','target':facts+'::RawInterruptStackFrame.reserved_1/reserved_2','meaning':'Exact six bytes in each gap preserved by codec; constructor zeroes both.'},
]
m['representation_notes']=['Canonical X86IdtGate retained; selected align16 differs from measured upstream Entry align4.','DetachedIdt semantic storage is not a native4096-byte layout claim; explicit byte codecs define serialized4096 bytes.','Raw saved-frame contents preserve bits without validity or return authority.','SelectorErrorCode mapping reused from completed descriptor slice.']
p=ROOT/'source/drivers/facts/x86_interrupts-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('inventory changed')
else:p.write_text(text)
print(shared.check(m,checkout))
