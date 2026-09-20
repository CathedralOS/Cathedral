#!/usr/bin/env python3
"""Finite pure-body cases; range expectations distinguish pin from AMD count encoding."""
from pathlib import Path
import sys,json
HERE=Path(__file__).resolve().parent
s=['module cases;','use x86_values::tlb_operands;','use x86_values::tlb_operands::PcidResult;','use x86_values::tlb_operands::BroadcastResult;','use x86_values::tlb_operands::BroadcastRequest;','use x86_values::tlb_operands::PageSelection;','use x86_values::tlb_operands::PcidSelection;','use x86_values::tlb_operands::AsidSelection;','use x86_values::tlb_operands::RangeStep;','use facts::x86_tlb_operands::Pcid;','use facts::x86_tlb_operands::InvlpgbLimits;']
s+=['machine pcid_is(result: PcidResult, expected: u16, valid: bool) -> bool { transition result { PcidResult::Valid { pcid } -> (valid && tlb_operands::pcid_value(pcid) == expected) PcidResult::TooBig { value } -> (!valid && value == expected) } }']
s+=['pub machine pcid_cases() -> bool { '+' && '.join('pcid_is(tlb_operands::pcid_new('+str(n)+'), '+str(n)+', '+str(n<4096).lower()+')' for n in [0,1,2,63,64,255,256,4094,4095,4096,4097,8191,32767,65535])+' }']
s+=['machine regs_are(value: BroadcastResult, rax: u64, ecx: u32, edx: u32) -> bool { transition value { BroadcastResult::Ready { registers } -> (registers.rax == rax && registers.ecx == ecx && registers.edx == edx) _ -> (false) } }']
for bits in range(64):
 page='PageSelection::Pages { address: 305418240, extra_count: 2, stride: 4096 }' if bits&1 else 'PageSelection::All'
 pcid='PcidSelection::One { pcid: Pcid { value: 123 } }' if bits&2 else 'PcidSelection::Any';asid='AsidSelection::One { value: 42 }' if bits&4 else 'AsidSelection::Any'
 s+=['machine broadcast_'+str(bits)+'() -> bool {',f'let request: BroadcastRequest = BroadcastRequest {{ pages: {page}, pcid: {pcid}, asid: {asid}, include_global: {str(bool(bits&8)).lower()}, final_only: {str(bool(bits&16)).lower()}, include_nested: {str(bool(bits&32)).lower()} }};', 'let result: BroadcastResult = tlb_operands::broadcast_prepare(request, InvlpgbLimits { count_max: 65535, nested: true, nasid: 65536 });', f'regs_are(result, {(305418240 if bits&1 else 0)|bits}, {2 if bits&1 else 0}, {((123<<16) if bits&2 else 0)|(42 if bits&4 else 0)})','}']
s+=['machine large_broadcast() -> bool {','let address: u64 = 0xffff800000000000;','let expected: u64 = 0xffff80000000003f;','let r0: BroadcastRequest = tlb_operands::broadcast_default();','let r1: BroadcastRequest = tlb_operands::with_pages(r0, PageSelection::Pages { address: address, extra_count: 65535, stride: 2097152 });','let r2: BroadcastRequest = tlb_operands::with_pcid(r1, Pcid { value: 4095 });','let r3: BroadcastRequest = tlb_operands::with_asid(r2, 65535);','let r4: BroadcastRequest = tlb_operands::with_global(r3);','let r5: BroadcastRequest = tlb_operands::with_final_only(r4);','let r6: BroadcastRequest = tlb_operands::with_nested(r5);','let result: BroadcastResult = tlb_operands::broadcast_prepare(r6, InvlpgbLimits { count_max: 65535, nested: true, nasid: 65536 });','regs_are(result, expected, 2147549183, 268435455)','}']
s+=['pub machine broadcast_cases() -> bool { '+' && '.join('broadcast_'+str(i)+'()' for i in range(64))+' && large_broadcast() }']
# Explicit expected tuples(extra, advance, next), independently reviewable.
H=0xffff800000000000;L=0x7ffffffff000
cases=[
(0x1000,0x4000,4096,0,(0,1,0x2000),(0,1,0x2000)),
(0x1000,0x4000,4096,1,(1,1,0x2000),(1,2,0x3000)),
(0x1000,0x4000,4096,2,(2,2,0x3000),(2,3,0x4000)),
(0x1000,0x4000,4096,65535,(3,3,0x4000),(2,3,0x4000)),
(0x1000,0x2000,4096,15,(1,1,0x2000),(0,1,0x2000)),
(L,H+4096,4096,15,(1,1,H),(0,1,H)),
(H,H+16384,4096,1,(1,1,H+4096),(1,2,H+8192)),
(0,65536*4096,4096,65535,(65535,65535,65535*4096),(65535,65536,65536*4096)),
(0,3*2097152,2097152,2,(2,2,2*2097152),(2,3,3*2097152)),
(0x7fffffe00000,H+2097152,2097152,15,(1,1,H),(0,1,H)),
(4096,4096,4096,1,None,None),(8192,4096,4096,1,None,None),
(1,8192,4096,1,'invalid','invalid'),(0,1073741824,1073741824,1,'invalid','invalid'),(0x800000000000,H,4096,1,'invalid','invalid')]
s+=['machine chunk_is(result: RangeStep, address: u64, extra: u16, advance: u32, next: u64) -> bool { transition result { RangeStep::Chunk { address, extra_count, advance, next } -> fields(address, extra_count, advance, next) _ -> (false) } state fields(a: u64, c: u16, n: u32, end: u64) { a == address && c == extra && n == advance && end == next } }']
# Avoid binding collisions between expected inputs and pattern fields.
s[-1]='machine chunk_is(result: RangeStep, want_address: u64, want_extra: u16, want_advance: u32, want_next: u64) -> bool { transition result { RangeStep::Chunk { address, extra_count, advance, next } -> (address == want_address && extra_count == want_extra && advance == want_advance && next == want_next) _ -> (false) } }'
for i,(start,end,stride,maximum,pin,arch) in enumerate(cases):
 s += [f'machine range_{i}() -> bool {{',f'let start: u64 = {start};',f'let end: u64 = {end};',f'let old: RangeStep = tlb_operands::pinned_range_step(start, end, {stride}, {maximum});',f'let new: RangeStep = tlb_operands::architectural_range_step(start, end, {stride}, {maximum});']
 checks=[]
 for name,expected in [('old',pin),('new',arch)]:
  if expected is None:checks.append(name+' in RangeStep::Done')
  elif expected=='invalid':checks.append(name+' in RangeStep::InvalidRange')
  else:
   s.append(f'let {name}_next: u64 = {expected[2]};');checks.append(f'chunk_is({name}, start, {expected[0]}, {expected[1]}, {name}_next)')
 s+=[' && '.join(checks),'}']
s+=['pub machine range_cases() -> bool { '+' && '.join('range_'+str(i)+'()' for i in range(len(cases)))+' && tlb_operands::architectural_page_count(65535) == 65536 && tlb_operands::pinned_advance(65535) == 65535 }']
outputs={HERE/'cases.omg':'\n'.join(s)+'\n',HERE/'range-cases.json':json.dumps(cases,indent=2)+'\n'}
for p,text in outputs.items():
 if '--check' in sys.argv:
  if not p.exists() or p.read_text()!=text:raise SystemExit('fixture drift: '+str(p))
 else:p.write_text(text)
print('14PCID boundaries,65broadcast cases,15pinned/architectural range pairs generated')
