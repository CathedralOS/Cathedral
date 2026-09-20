#!/usr/bin/env python3
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent;schema=json.loads((HERE/'schema.json').read_text())
lines=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Independent literal expectations; actual imported helper bodies are evaluated.','module cases;','use x86_values::interrupts;','use x86_values::interrupts::StackIndex;','use x86_values::interrupts::IdtIndexKind;','use x86_values::interrupts::ExceptionVectorCheck;','use x86_values::interrupt_bytes;','use facts::x86_interrupts;','use facts::x86_interrupts::EntryOptions;','use facts::x86_interrupts::RawInterruptStackFrame;','use facts::x86_interrupts::PageFaultErrorCode;','use facts::x86_registers::SegmentSelector;','use facts::x86_registers::RFlags;','use facts::x86_exception_vectors;']
def groups(label,checks,chunk=8):
 names=[]
 for start in range(0,len(checks),chunk):
  name=label+'_'+str(start//chunk);names.append(name+'()');lines.append('machine '+name+'() -> bool { '+' &&\n'.join(checks[start:start+chunk])+' }')
 lines.append('pub machine '+label+'() -> bool { '+' && '.join(names)+' }')
flags=[f'x86_interrupts::PAGE_FAULT_{f["name"]}.raw == {f["value"]}' for f in schema['flags']]
flags+=['x86_interrupts::PAGE_FAULT_KNOWN_BITS == 2147516671','interrupts::page_fault_known(2147516671)','!interrupts::page_fault_known(256)','interrupts::page_fault_truncate(0xffffffffffffffff).raw == 2147516671','interrupts::page_fault_contains(PageFaultErrorCode { raw: 3 }, PageFaultErrorCode { raw: 1 })','!interrupts::page_fault_contains(PageFaultErrorCode { raw: 1 }, PageFaultErrorCode { raw: 2 })']
groups('flag_cases',flags)
lines+=['''machine stack_is(value: StackIndex, expected: u8) -> bool {
 transition value { StackIndex::Dedicated { index } -> (index == expected) _ -> (false) }
}
machine option_case(index: u8 [0..=6], dpl: u8 [0..=3], present: bool, disable: bool, expected: u16) -> bool {
 let minimal: EntryOptions = interrupts::options_minimal();
 let selected: EntryOptions = interrupts::options_set_selector(minimal, SegmentSelector { raw: 43981 });
 let stack: EntryOptions = interrupts::options_set_stack_index(selected, index);
 let privilege: EntryOptions = interrupts::options_set_privilege(stack, dpl);
 let visibility: EntryOptions = interrupts::options_set_present(privilege, present);
 let result: EntryOptions = interrupts::options_disable_interrupts(visibility, disable);
 result.selector.raw == 43981 && result.bits == expected && interrupts::options_present(result) == present &&
 interrupts::options_privilege(result) == dpl && stack_is(interrupts::options_stack_index(result), index)
}''']
checks=['interrupts::options_stack_index(interrupts::options_minimal()) in StackIndex::Current']
for index in range(7):
 for dpl in range(4):
  for present in [False,True]:
   for disable in [False,True]:
    expected=0xe00|(index+1)|(dpl<<13)|(0x8000 if present else 0)|(0 if disable else 0x100)
    checks.append(f'option_case({index}, {dpl}, {str(present).lower()}, {str(disable).lower()}, {expected})')
groups('option_cases',checks)
lines+=['''machine index_code(value: IdtIndexKind) -> u8 {
 transition value { IdtIndexKind::Available -> (0) IdtIndexKind::Reserved -> (1) IdtIndexKind::ErrorCode -> (2) IdtIndexKind::Diverging -> (3) }
}
machine vector_check_is(value: ExceptionVectorCheck, expected: u8, valid: bool) -> bool {
 transition value { ExceptionVectorCheck::Known { value } -> (value == expected && valid) ExceptionVectorCheck::Invalid { value } -> (value == expected && !valid) }
}''']
checks=[];known={r['value'] for r in schema['exception_vectors']}
for vector in range(256):
 kind=1 if vector in [15,31,*range(22,28)] else 2 if vector in [8,*range(10,15),17,21,29,30] else 3 if vector==18 else 0
 valid=str(vector in known).lower();checks.append(f'(interrupts::exception_vector_known({vector}) == {valid} && vector_check_is(interrupts::exception_vector_check({vector}), {vector}, {valid}) && index_code(interrupts::idt_index_kind({vector})) == {kind})')
existing=['X86_EXCEPTION_DIVIDE_ERROR','X86_EXCEPTION_DEBUG','X86_EXCEPTION_NON_MASKABLE_INTERRUPT','X86_EXCEPTION_BREAKPOINT','X86_EXCEPTION_OVERFLOW','X86_EXCEPTION_BOUND_RANGE_EXCEEDED','X86_EXCEPTION_INVALID_OPCODE','X86_EXCEPTION_DEVICE_NOT_AVAILABLE','X86_EXCEPTION_DOUBLE_FAULT','X86_EXCEPTION_INVALID_TSS','X86_EXCEPTION_SEGMENT_NOT_PRESENT','X86_EXCEPTION_STACK_SEGMENT_FAULT','X86_EXCEPTION_GENERAL_PROTECTION','X86_EXCEPTION_PAGE_FAULT','X86_EXCEPTION_X87_FLOATING_POINT','X86_EXCEPTION_ALIGNMENT_CHECK','X86_EXCEPTION_MACHINE_CHECK','X86_EXCEPTION_SIMD_FLOATING_POINT','X86_EXCEPTION_VIRTUALIZATION','X86_EXCEPTION_CONTROL_PROTECTION','AMD_EXCEPTION_HYPERVISOR_INJECTION','AMD_EXCEPTION_VMM_COMMUNICATION','AMD_EXCEPTION_SECURITY']
assert len(existing)==len(schema['exception_vectors'])
checks +=[f'{name} == {row["value"]}' for name,row in zip(existing,schema['exception_vectors'])]
groups('vector_cases',checks)
names=[]
for name,row in schema['records'].items():
 fields={};expected=bytearray(row['size'])
 for field,ty,offset in row['fields']:
  if ty.startswith('['):
   values=list(range(offset+1,offset+7));fields[field]='['+', '.join(map(str,values))+']';expected[offset:offset+6]=bytes(values)
  else:
   width=2 if ty=='SegmentSelector' else 8 if ty=='RFlags' else int(ty[1:])//8
   value=int.from_bytes(bytes(range(offset+1,offset+width+1)),'little');fields[field]=str(value) if ty.startswith('u') else ty+' { raw: '+str(value)+' }';expected[offset:offset+width]=value.to_bytes(width,'little')
 fn='bytes_'+name;names.append(fn+'()');lines.append('machine '+fn+'() -> bool {')
 lines.append(f'let value: {name} = {name} {{ '+', '.join(f+': '+v for f,v in fields.items())+' };')
 lines.append(f'let encoded: [u8; {row["size"]}] = interrupt_bytes::encode_{name}(&value);')
 lines.append(f'let golden: [u8; {row["size"]}] = ['+', '.join(map(str,expected))+'];')
 lines.append(f'let decoded: {name} = interrupt_bytes::decode_{name}(&golden);')
 tests=[f'encoded[{i}] == {b}' for i,b in enumerate(expected)]
 for field,ty,_ in row['fields']:
  if ty.startswith('['):tests +=[f'decoded.{field}[{i}] == value.{field}[{i}]' for i in range(6)]
  elif ty in ('SegmentSelector','RFlags'):tests.append(f'decoded.{field}.raw == value.{field}.raw')
  else:tests.append(f'decoded.{field} == value.{field}')
 lines+=[' &&\n'.join(tests),'}']
lines.append('''machine frame_default() -> bool {
 let value: RawInterruptStackFrame = interrupts::frame_new(4096, SegmentSelector { raw: 8 }, RFlags { raw: 512 }, 8192, SegmentSelector { raw: 16 });
 value.instruction_pointer == 4096 && value.code_segment.raw == 8 && value.cpu_flags.raw == 512 && value.stack_pointer == 8192 && value.stack_segment.raw == 16 && '''+' && '.join(f'value.reserved_{n}[{i}] == 0' for n in [1,2] for i in range(6))+' }')
lines.append('pub machine record_byte_cases() -> bool { '+' && '.join(names+['frame_default()'])+' }')
text='\n'.join(lines)+'\n';p=HERE/'cases.omg'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('fixtures differ')
else:p.write_text(text)
print('112option combinations,256vector/index checks,23existing identity constants,full4/40byte record codecs')
