#!/usr/bin/env python3
"""Bounded wrapper assertions; expensive route calls stay in a separate fixture."""
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
imports='''use {x86_values}/mapper_conveniences.omg;
use {x86_values}/mapping_routes.omg;
use {x86_values}/addresses.omg;
use {x86_values}/translation.omg;
'''
s=[imports,'''machine number_is(value: |addresses|::NumberResult, expected: u64) -> bool { transition value { |addresses|::NumberResult::Value { value } -> (value == expected) _ -> (false) } }
machine request_is(value: |mapping_routes|::Request, expected_frame: u64, expected_flags: u64, expected_parent: u64) -> bool {
 transition value { |mapping_routes|::Request::Map { frame, flags, parent_flags } -> (frame == expected_frame && flags == expected_flags && parent_flags == expected_parent) _ -> (false) }
}
machine frame_is(value: |mapper_conveniences|::MappedFrameCheck, expected_address: u64, expected_size: u64) -> bool {
 transition value { |mapper_conveniences|::MappedFrameCheck::Value { frame } -> check(frame, expected_address, expected_size) _ -> (false) }
 state check(value: |mapper_conveniences|::MappedFrame, address: u64, size: u64) {
  let start: u64 = |mapper_conveniences|::frame_start(value);
  let length: u64 = |mapper_conveniences|::frame_size(value);
  start == address && length == size
 }
}
''']
checks=[]
for n,size in enumerate([4096,2097152,1073741824]):
 for i,flags in enumerate([0,1,2,4,7,128,256,0x12345007,0x8000000000000007]):
  name=f'default_{n}_{i}';checks.append(name+'()');s+=[f'machine {name}() -> bool {{',f'let flags: u64 = {flags};',f'let request: |mapping_routes|::Request = |mapper_conveniences|::default_map_request({size*3}, flags);',f'request_is(request, {size*3}, flags, {flags&7})','}']
 s+=[f'machine frame_{n}() -> bool {{',f'let checked: |mapper_conveniences|::MappedFrameCheck = |mapper_conveniences|::mapped_frame({size*2}, {size});',f'let observed: |mapper_conveniences|::MappedFrameCheck = |mapper_conveniences|::frame_from_translation(|translation|::Translation::Mapped {{ frame: {size*2}, size: {size}, offset: 1, flags: 0 }});',f'frame_is(checked, {size*2}, {size}) && frame_is(observed, {size*2}, {size}) && number_is(|mapper_conveniences|::identity_page({size*3}, {size}), {size*3})','}'];checks.append(f'frame_{n}()')
 for i,offset in enumerate([0,size-1,size,size+1]):
  name=f'translation_{n}_{i}';checks.append(name+'()');s+=[f'machine {name}() -> bool {{',f'let result: |addresses|::NumberResult = |mapper_conveniences|::translated_address_pinned(|translation|::Translation::Mapped {{ frame: {size*2}, size: {size}, offset: {offset}, flags: 0 }});',f'number_is(result, {size*2+offset})','}']
s+=['''machine invalids() -> bool {
 let bit47: u64 = 0x800000000000;
 let bad_identity: |addresses|::NumberResult = |mapper_conveniences|::identity_page(bit47, 4096);
 let unaligned: |addresses|::NumberResult = |mapper_conveniences|::identity_page(4097, 4096);
 let bad_size: |mapper_conveniences|::MappedFrameCheck = |mapper_conveniences|::mapped_frame(0, 8192);
 let bad_frame: |mapper_conveniences|::MappedFrameCheck = |mapper_conveniences|::mapped_frame(4097, 4096);
 let unmapped: |mapper_conveniences|::MappedFrameCheck = |mapper_conveniences|::frame_from_translation(|translation|::Translation::NotMapped);
 let overflow: |addresses|::NumberResult = |mapper_conveniences|::translated_address_pinned(|translation|::Translation::Mapped { frame: 0xffffffffff000, size: 4096, offset: 4096, flags: 0 });
 let empty: |addresses|::NumberResult = |mapper_conveniences|::translated_address_pinned(|translation|::Translation::NotMapped);
 let strict: |addresses|::NumberResult = |translation|::translated_address(|translation|::Translation::Mapped { frame: 8192, size: 4096, offset: 4096, flags: 0 });
 bad_identity in |addresses|::NumberResult::Rejected && unaligned in |addresses|::NumberResult::Rejected && bad_size in |mapper_conveniences|::MappedFrameCheck::Rejected && bad_frame in |mapper_conveniences|::MappedFrameCheck::Rejected && unmapped in |mapper_conveniences|::MappedFrameCheck::Rejected && overflow in |addresses|::NumberResult::Rejected && empty in |addresses|::NumberResult::Rejected && strict in |addresses|::NumberResult::Rejected
}
machine captured() -> bool {
 let address: |addresses|::NumberResult = |mapper_conveniences|::translate_address_from_words(0x4321, 4097, 8193, 12289, 36871);
 let absent: |addresses|::NumberResult = |mapper_conveniences|::translate_address_from_words(0x4321, 0, 0, 0, 0);
 let metadata: |addresses|::NumberResult = |mapper_conveniences|::translate_address_from_words(0xabc, 4097, 8193, 12289, 4096);
 number_is(address, 37665) && absent in |addresses|::NumberResult::Rejected && number_is(metadata, 6844)
}
''']
checks+=['invalids()','captured()'];s+=['machine test_result() -> i32 { transition '+' && '.join(checks)+' { true -> (0) _ -> (1) } }','const TEST_RESULT: i32 = test_result();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(TEST_RESULT); }']
p=HERE/'main.omg';text='\n'.join(s)+'\n'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('generated wrapper fixture drift')
else:p.write_text(text)
print('27default requests,3frame/identity groups,12translation offsets and malformed/captured cases')
