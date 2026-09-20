#!/usr/bin/env python3
"""Bounded wrapper assertions; expensive route calls stay in a separate fixture."""
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
imports='''use x86_values::mapper_conveniences;
use x86_values::mapper_conveniences::MappedFrame;
use x86_values::mapper_conveniences::MappedFrameCheck;
use x86_values::mapping_routes::Request;
use x86_values::addresses::NumberResult;
use x86_values::translation;
use x86_values::translation::Translation;
'''
s=[imports,'''machine number_is(value: NumberResult, expected: u64) -> bool { transition value { NumberResult::Value { value } -> (value == expected) _ -> (false) } }
machine request_is(value: Request, expected_frame: u64, expected_flags: u64, expected_parent: u64) -> bool {
 transition value { Request::Map { frame, flags, parent_flags } -> (frame == expected_frame && flags == expected_flags && parent_flags == expected_parent) _ -> (false) }
}
machine frame_is(value: MappedFrameCheck, expected_address: u64, expected_size: u64) -> bool {
 transition value { MappedFrameCheck::Value { frame } -> check(frame, expected_address, expected_size) _ -> (false) }
 state check(value: MappedFrame, address: u64, size: u64) {
  let start: u64 = mapper_conveniences::frame_start(value);
  let length: u64 = mapper_conveniences::frame_size(value);
  start == address && length == size
 }
}
''']
checks=[]
for n,size in enumerate([4096,2097152,1073741824]):
 for i,flags in enumerate([0,1,2,4,7,128,256,0x12345007,0x8000000000000007]):
  name=f'default_{n}_{i}';checks.append(name+'()');s+=[f'machine {name}() -> bool {{',f'let flags: u64 = {flags};',f'let request: Request = mapper_conveniences::default_map_request({size*3}, flags);',f'request_is(request, {size*3}, flags, {flags&7})','}']
 s+=[f'machine frame_{n}() -> bool {{',f'let checked: MappedFrameCheck = mapper_conveniences::mapped_frame({size*2}, {size});',f'let observed: MappedFrameCheck = mapper_conveniences::frame_from_translation(Translation::Mapped {{ frame: {size*2}, size: {size}, offset: 1, flags: 0 }});',f'frame_is(checked, {size*2}, {size}) && frame_is(observed, {size*2}, {size}) && number_is(mapper_conveniences::identity_page({size*3}, {size}), {size*3})','}'];checks.append(f'frame_{n}()')
 for i,offset in enumerate([0,size-1,size,size+1]):
  name=f'translation_{n}_{i}';checks.append(name+'()');s+=[f'machine {name}() -> bool {{',f'let result: NumberResult = mapper_conveniences::translated_address_pinned(Translation::Mapped {{ frame: {size*2}, size: {size}, offset: {offset}, flags: 0 }});',f'number_is(result, {size*2+offset})','}']
s+=['''machine invalids() -> bool {
 let bit47: u64 = 0x800000000000;
 let bad_identity: NumberResult = mapper_conveniences::identity_page(bit47, 4096);
 let unaligned: NumberResult = mapper_conveniences::identity_page(4097, 4096);
 let bad_size: MappedFrameCheck = mapper_conveniences::mapped_frame(0, 8192);
 let bad_frame: MappedFrameCheck = mapper_conveniences::mapped_frame(4097, 4096);
 let unmapped: MappedFrameCheck = mapper_conveniences::frame_from_translation(Translation::NotMapped);
 let overflow: NumberResult = mapper_conveniences::translated_address_pinned(Translation::Mapped { frame: 0xffffffffff000, size: 4096, offset: 4096, flags: 0 });
 let empty: NumberResult = mapper_conveniences::translated_address_pinned(Translation::NotMapped);
 let strict: NumberResult = translation::translated_address(Translation::Mapped { frame: 8192, size: 4096, offset: 4096, flags: 0 });
 bad_identity in NumberResult::Rejected && unaligned in NumberResult::Rejected && bad_size in MappedFrameCheck::Rejected && bad_frame in MappedFrameCheck::Rejected && unmapped in MappedFrameCheck::Rejected && overflow in NumberResult::Rejected && empty in NumberResult::Rejected && strict in NumberResult::Rejected
}
machine captured() -> bool {
 let address: NumberResult = mapper_conveniences::translate_address_from_words(0x4321, 4097, 8193, 12289, 36871);
 let absent: NumberResult = mapper_conveniences::translate_address_from_words(0x4321, 0, 0, 0, 0);
 let metadata: NumberResult = mapper_conveniences::translate_address_from_words(0xabc, 4097, 8193, 12289, 4096);
 number_is(address, 37665) && absent in NumberResult::Rejected && number_is(metadata, 6844)
}
''']
checks+=['invalids()','captured()'];s+=['machine test_result() -> i32 { transition '+' && '.join(checks)+' { true -> (0) _ -> (1) } }','const TEST_RESULT: i32 = test_result();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(TEST_RESULT); }']
p=HERE/'main.omg';text='\n'.join(s)+'\n'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('generated wrapper fixture drift')
else:p.write_text(text)
print('27default requests,3frame/identity groups,12translation offsets and malformed/captured cases')
