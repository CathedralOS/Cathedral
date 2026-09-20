#!/usr/bin/env python3
"""Deterministic actual-Rust witnesses and Omega PTE/index/level expectations."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
rows=[]
def add(body,rust):rows.append(dict(body=body,rust=rust))
MAX=2**64-1;MASK=0x000ffffffffff000
for word in [0,MAX,0xaaaaaaaaaaaaaaaa,0x5555555555555555]+[1<<bit for bit in range(64)]:
 checks=['page_entries::encode(&value) == '+str(word),'page_entries::is_unused(&value) == '+str(word==0).lower(),f'page_entries::address({word}) == {word&MASK}',f'page_entries::flags({word}) == {word&~MASK}']
 fields=[('present',0,1),('writable',1,1),('user',2,1),('write_through',3,1),('cache_disable',4,1),('accessed',5,1),('dirty',6,1),('page_size_or_pat',7,1),('global',8,1),('software_low',9,3),('frame_number',12,40),('software_high',52,7),('protection_key',59,4),('no_execute',63,1)]
 for name,shift,width in fields:
  expected=(word>>shift)&((1<<width)-1)
  checks.append(f'value.{name} == '+(str(bool(expected)).lower() if width==1 else str(expected)))
 body=f'let value: X86PageTableEntry = page_entries::decode({word});\n    transition '+' && '.join(checks)+' { true -> (0) _ -> (1) }'
 rust=f'let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new({word&MASK}),PageTableFlags::from_bits_retain({word&~MASK}));assert_eq!(e.addr().as_u64(),{word&MASK});assert_eq!(e.flags().bits(),{word&~MASK});assert_eq!(e.is_unused(),{str(word==0).lower()});'
 add(body,rust)
def num(call,expected,rust):
 check='rejected(value)' if expected is None else f'equal(value, {expected})'
 add(f'let value: NumberResult = page_entries::{call};\n    transition {check} {{ true -> (0) _ -> (1) }}',rust)
for base,bits in [(0,0),(4096,3),(MASK,2**63|127),(4096,0x2000|1),(1,3),(2**52,1)]:
 valid=base<=MASK and base%4096==0
 expected=base|bits if valid else None
 num(f'set_address({base}, {bits})',expected,f'assert_eq!(caught(||{{let mut e=PageTableEntry::new(); e.set_addr(PhysAddr::new({base}),PageTableFlags::from_bits_retain({bits})); e.addr().as_u64()|e.flags().bits()}}),'+('None' if expected is None else f'Some({expected})')+');')
for word in [0,128,1,129,0x12345001,MAX]:
 expected='NotPresent' if not word&1 else 'HugeFrame' if word&128 else 'Value'
 if expected=='Value':body=f'let result: FrameResult = page_entries::frame({word}); transition result {{ FrameResult::Value {{ address }} -> finish(address) _ -> (1) }} state finish(address: u64) -> i32 {{ transition address == {word&MASK} {{ true -> (0) _ -> (1) }} }}'
 else:body=f'let result: FrameResult = page_entries::frame({word}); transition result in FrameResult::{expected} {{ true -> (0) _ -> (1) }}'
 rust=f'let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new({word&MASK}),PageTableFlags::from_bits_retain({word&~MASK}));'
 rust+=f'assert_eq!(e.frame().map(|p|p.start_address().as_u64()),'+(f'Ok({word&MASK})' if expected=='Value' else 'Err(FrameError::'+('FrameNotPresent' if expected=='NotPresent' else 'HugeFrame')+')')+');'
 add(body,rust)
for base,bits in [(0,0),(4096,1),(4096,129),(0,MAX),(MASK,2**63|1)]:
 e=base|bits if not bits&128 else None
 num(f'set_frame({base}, {bits})',e,f'assert_eq!(caught(||{{let mut v=PageTableEntry::new();v.set_frame(PhysFrame::containing_address(PhysAddr::new({base})),PageTableFlags::from_bits_retain({bits}));v.addr().as_u64()|v.flags().bits()}}),'+('None' if e is None else f'Some({e})')+');')
for word,bits in [(MAX,0),(4097,3),(4097,0x2000|1),(0,MAX)]:
 expected=(word&MASK)|bits
 add(f'transition page_entries::set_flags({word}, {bits}) == {expected} {{ true -> (0) _ -> (1) }}',f'let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new({word&MASK}),PageTableFlags::from_bits_retain({word&~MASK}));e.set_flags(PageTableFlags::from_bits_retain({bits}));assert_eq!(e.addr().as_u64()|e.flags().bits(),{expected});')
for kind,limit,ty in [('index',512,'PageTableIndex'),('offset',4096,'PageOffset')]:
 for value in [0,1,limit-1,limit,65535]:
  e=value if value<limit else None
  num(f'{kind}_checked({value})',e,f'assert_eq!(caught(||u64::from({ty}::new({value}))),'+('None' if e is None else f'Some({e})')+');')
  add(f'transition page_entries::{kind}_truncate({value}) == {value%(limit)} {{ true -> (0) _ -> (1) }}',f'assert_eq!(u64::from({ty}::new_truncate({value})),{value%limit});')
for start,count,back in [(0,0,False),(0,511,False),(511,511,True),(511,1,False),(0,1,True),(511,MAX,False),(511,MAX,True)]:
 value=start-count if back else start+count;e=value if 0<=value<512 else None
 direction='backward' if back else 'forward'
 num(f'index_step({start}, {count}, {str(back).lower()})',e,f'assert_eq!(Step::{direction}_checked(PageTableIndex::new({start}),{count}).map(u64::from),'+('None' if e is None else f'Some({e})')+');')
 add(f'let value: OverflowingStep = page_entries::index_overflowing({start}, {count}, {str(back).lower()}); transition value.value == {start if e is None else e} && value.overflow == {str(e is None).lower()} {{ true -> (0) _ -> (1) }}',f'let (v,o)=Step::{direction}_overflowing(PageTableIndex::new({start}),{count});assert_eq!((u64::from(v),o),({start if e is None else e},{str(e is None).lower()}));')
for a,b in [(0,0),(0,511),(511,0),(511,511)]:
 num(f'index_distance({a}, {b})',b-a if b>=a else None,f'assert_eq!(Step::steps_between(&PageTableIndex::new({a}),&PageTableIndex::new({b})).1,'+('None' if b<a else f'Some({b-a})')+');')
for level,name in enumerate(['One','Two','Three','Four'],1):
 for higher in [False,True]:
  e=level+1 if higher else level-1;e=e if 1<=e<=4 else None
  num(f'next_level({level}, {str(higher).lower()})',e,f'assert_eq!(PageTableLevel::{name}.next_{"higher" if higher else "lower"}_level().map(|v|v as u64),'+('None' if e is None else f'Some({e})')+');')
 for whole in [False,True]:
  e=1<<(12+9*(level if whole else level-1))
  num(f'level_alignment({level}, {str(whole).lower()})',e,f'assert_eq!(PageTableLevel::{name}.{"table" if whole else "entry"}_address_space_alignment(),{e});')
for call in ['index_checked(18446744073709551615)','offset_checked(18446744073709551615)','index_step(512, 0, false)','index_distance(512, 512)','next_level(0, true)','next_level(5, false)','level_alignment(0, false)','level_alignment(5, true)']:num(call,None,None)
def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale generated fixture',path)
 else:path.write_text(text)
prefix='// SPDX-License-Identifier: MIT OR Apache-2.0\nuse x86::page_entries;\nuse x86::page_entries::FrameResult;\nuse x86::addresses::NumberResult;\nuse x86::addresses::OverflowingStep;\nuse facts::x86_page_table_entry;\nmachine equal(result: NumberResult, expected: u64) -> bool { transition result { NumberResult::Value { value } -> (value == expected) _ -> (false) } }\nmachine rejected(result: NumberResult) -> bool { result in NumberResult::Rejected }\n'
write(HERE/'cases.json',json.dumps(rows,indent=2)+'\n')
write(HERE/'main.omg',prefix+'\n'.join(f'machine case_{i}() -> i32 {{ {row["body"]} }}' for i,row in enumerate(rows))+'\ndata Main {}\nmachine Main::main(&mut self) {}\n')
rust='#![feature(step_trait)]\nuse core::iter::Step;\nuse x86_64::PhysAddr;\nuse x86_64::structures::paging::{PhysFrame,PageTableFlags,PageTableIndex,PageOffset};\nuse x86_64::structures::paging::page_table::{PageTableEntry,FrameError,PageTableLevel};\nfn caught(f: impl FnOnce()->u64+std::panic::UnwindSafe)->Option<u64>{std::panic::catch_unwind(f).ok()}\nfn main(){std::panic::set_hook(Box::new(|_|{}));\n'
rust+='\n'.join('{'+row['rust']+'}' for row in rows if row['rust'])
rust+=f'\nprintln!("{sum(bool(row["rust"]) for row in rows)} actual pinned PTE/index/level witnesses passed");\n}}\n'
write(HERE/'src/main.rs',rust)
print(len(rows),'Omega cases;',sum(bool(r['rust']) for r in rows),'actual Rust witnesses')
