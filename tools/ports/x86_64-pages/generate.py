#!/usr/bin/env python3
"""Reviewed numeric fixtures for pinned page/frame algorithms and checked deviations."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
rows=[]
def add(call,expected,rust=None,rust_expected='same'):
 rows.append(dict(call=call,expected=expected,rust=rust,rust_expected=expected if rust_expected=='same' else rust_expected))
def literal(x):return str(x).lower() if isinstance(x,bool) else str(x)
def call(name,*args):return name+'('+', '.join(literal(x) for x in args)+')'
MAX=2**64-1;PHY=2**52-1;LOW=2**47-1;HIGH=2**64-2**47;DENSE=2**48-1
for size,typ in [(4096,'Size4KiB'),(2**21,'Size2MiB'),(2**30,'Size1GiB')]:
 page=lambda x:f'Page::<{typ}>::containing_address(VirtAddr::new({x}))'
 frame=lambda x:f'PhysFrame::<{typ}>::containing_address(PhysAddr::new({x}))'
 for physical in [False,True]:
  make=frame if physical else page;limit=PHY if physical else MAX
  for x in [0,1,0xdeadbeaf,limit]:
   e=x&~(size-1)
   add(call('containing',x,size,physical),e,f'Some({make(x)}.start_address().as_u64())')
   add(call('from_start',x,size,physical),x if x%size==0 else None,f'{"PhysFrame" if physical else "Page"}::<{typ}>::from_start_address({"PhysAddr" if physical else "VirtAddr"}::new({x})).ok().map(|p| p.start_address().as_u64())')
  for backwards in [False,True]:
   x=size*3;n=2;e=x-size*n if backwards else x+size*n
   add(call('arithmetic',x,size,n,backwards,physical),e,f'caught(|| ({make(x)} {"-" if backwards else "+"} {n}u64).start_address().as_u64())')
  for inclusive in [False,True]:
   start=0;end=3*size;count=4 if inclusive else 3
   rng=f'{"PhysFrame" if physical else "Page"}::range'+('_inclusive' if inclusive else '')+f'({make(start)}, {make(end)})'
   add(call('range_count',start,end,size,inclusive,physical),count,f'Some({rng}.len())')
   add(call('range_bytes',start,end,size,inclusive,physical),count*size,f'Some({rng}.size())')
   for index in [0,count-1,count]:
    for reverse in [False,True]:
     expected=None if index>=count else (count-1-index if reverse else index)*size
     add(call('range_at',start,end,size,inclusive,physical,index,reverse),expected,f'{rng}.nth'+('_back' if reverse else '')+f'({index}).map(|p| p.start_address().as_u64())')
  last=limit&~(size-1)
  rng=f'{"PhysFrame" if physical else "Page"}::range_inclusive({make(last)}, {make(last)})'
  add(call('range_at',last,last,size,True,physical,0,False),last,f'{rng}.next().map(|p| p.start_address().as_u64())')
  add(call('range_at',last,last,size,True,physical,1,False),None,f'{rng}.nth(1).map(|p| p.start_address().as_u64())')
  add(call('range_count',size,0,size,False,physical),0,f'Some({"PhysFrame" if physical else "Page"}::range({make(size)}, {make(0)}).len())')
 for pfn in [0,0x123,PHY//size,PHY//size+1,MAX]:
  add(call('frame_from_pfn',pfn,size),pfn*size if pfn<=PHY//size else None,f'PhysFrame::<{typ}>::try_from_pfn({pfn}).ok().map(|p| p.start_address().as_u64())')
 add(call('frame_pfn',0xc0000000,size),0xc0000000//size,f'Some({frame(0xc0000000)}.pfn())')
 p4,p3,p2,p1=511,123,(45 if size<2**30 else 0),(67 if size==4096 else 0)
 raw=(p4<<39)|(p3<<30)|(p2<<21)|(p1<<12);expected=raw|0xffff000000000000
 suffix={4096:'',2**21:'_2mib',2**30:'_1gib'}[size];indices=[p4,p3]+([p2] if size<2**30 else [])+([p1] if size==4096 else [])
 add(call('from_indices',p4,p3,p2,p1,size),expected,f'Some(Page::<{typ}>::from_page_table_indices{suffix}('+', '.join(f'PageTableIndex::new({i})' for i in indices)+').start_address().as_u64())')
 low_last=LOW&~(size-1)
 for backwards,start in [(False,low_last),(True,HIGH)]:
  add(call('step',start,size,1,backwards),low_last if backwards else HIGH,f'Step::{"backward" if backwards else "forward"}_checked({page(start)}, 1).map(|p|p.start_address().as_u64())')
  add(call('arithmetic',start,size,1,backwards,False),None,f'caught(|| ({page(start)} {"-" if backwards else "+"} 1u64).start_address().as_u64())')
 add(call('steps_between',low_last,HIGH,size),1,f'Step::steps_between(&{page(low_last)}, &{page(HIGH)}).1.map(|n|n as u64)')
 add(call('difference',HIGH,low_last,size,False),(HIGH-low_last)//size,f'Some({page(HIGH)} - {page(low_last)})')
 # Strict range admission rejects a gap-spanning range before any iteration.
 add(call('range_count',low_last,HIGH,size,False,False),None)
 # Indexed singleton lookup avoids the pinned iterator's post-selection gap panic.
 add(call('range_at',low_last,low_last,size,True,False,0,False),low_last,f'caught(|| Page::range_inclusive({page(low_last)}, {page(low_last)}).next().unwrap().start_address().as_u64())',None)
 add(call('range_at',HIGH,HIGH,size,True,False,0,True),HIGH,f'caught(|| Page::range_inclusive({page(HIGH)}, {page(HIGH)}).next_back().unwrap().start_address().as_u64())',None)
for c,e in [(call('containing',0,0,False),None),(call('containing',0,8192,False),None),(call('from_start',2**47,4096,False),None),(call('from_start',2**52,4096,True),None),(call('frame_from_pfn',0,0),None),(call('from_indices',512,0,0,0,4096),None),(call('from_indices',0,0,0,1,2**21),None),(call('from_indices',0,0,1,0,2**30),None),(call('arithmetic',0,4096,MAX,False,False),None),(call('step',0,4096,MAX,False),None),(call('range_count',1,4096,4096,False,False),None)]:add(c,e)
# All pinned 64-bit forward/backward/distance table cases.
for backwards,values in [(False,[(0,0),(0,1),(4096,1),(0x7ffffffff000,1),(HIGH,1),(MAX&~4095,1),(0x7ffffffff000,0x123456789),(0x7ffffffff000,0x800000000),(0x7ffffff00000,0x8000000ff),(0x7ffffff00000,0x800000100),(0x7ffffffff000,0x800000001),(0,0x100000)]),(True,[(0,0),(0,1),(4096,1),(HIGH,1),(HIGH+4096,1),(0xffff923456788000,0x123456789),(HIGH,0x800000000),(HIGH,0x7ffffff01),(HIGH,0x800000001),(2**32,0x100000)])]:
 for x,n in values:
  rank=(x&DENSE)+(-n if backwards else n)*4096;e=None if not 0<=rank<=DENSE else rank if rank<=LOW else rank|0xffff000000000000
  add(call('step',x,4096,n,backwards),e,f'Step::{"backward" if backwards else "forward"}_checked(Page::<Size4KiB>::containing_address(VirtAddr::new({x})), {n}).map(|p|p.start_address().as_u64())')
for a,b in [(0,0),(0,4096),(4096,0),(4096,4096),(0x7ffffffff000,HIGH),(HIGH,0x7ffffffff000),(HIGH,HIGH),(HIGH,HIGH+4096),(HIGH+4096,HIGH),(HIGH+4096,HIGH+4096),(0,2**32),(0,2**44)]:
 e=((b&DENSE)-(a&DENSE))//4096 if b>=a else None
 add(call('steps_between',a,b,4096),e,f'Step::steps_between(&Page::<Size4KiB>::containing_address(VirtAddr::new({a})), &Page::<Size4KiB>::containing_address(VirtAddr::new({b}))).1.map(|n|n as u64)')
def write(path,text):
 if '--check' in sys.argv:
  if not path.is_file() or path.read_text()!=text:raise SystemExit(f'generated fixture differs: {path}')
 else:path.write_text(text)
source=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Modified pinned page/frame test expectations with explicit checked deviations.','use x86::pages;','use x86::addresses::NumberResult;','use extras;','machine equal(result: NumberResult, expected: u64) -> bool { transition result { NumberResult::Value { value } -> (value == expected) _ -> (false) } }','machine rejected(result: NumberResult) -> bool { result in NumberResult::Rejected }']
groups=[]
for start in range(0,len(rows),8):
 name=f'group_{start}';groups.append(name);source.append(f'machine {name}() -> i32 {{')
 for i,row in enumerate(rows[start:start+8],start):
  source.append(f'    let result{i}: NumberResult = pages::{row["call"]};')
  check=f'rejected(result{i})' if row['expected'] is None else f'equal(result{i}, {row["expected"]})'
  source.append(f'    let passed{i}: bool = {check};')
 source.append('    transition '+' && '.join(f'passed{i}' for i in range(start,min(start+8,len(rows))))+' { true -> (0) _ -> (1) }\n}')
source.append('machine test_result() -> i32 {')
for name in groups:source.append(f'    let {name}_ok: bool = {name}() == 0;')
source.append('    let extras_ok: bool = extras::result() == 0;')
source.append('    transition extras_ok && '+' && '.join(name+'_ok' for name in groups)+' { true -> (0) _ -> (1) }\n}')
source += ['const TEST_RESULT: i32 = test_result();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(TEST_RESULT); }\n']
write(HERE/'main.omg','\n'.join(source));write(HERE/'vectors.json',json.dumps({'scope':'numeric expected results; explicit Rust deviations recorded per case','cases':rows},indent=2)+'\n')
rust=['#![feature(step_trait)]','use core::iter::Step;','use x86_64::{VirtAddr,PhysAddr};','use x86_64::structures::paging::{Page,PhysFrame,Size4KiB,Size2MiB,Size1GiB,PageTableIndex};','fn caught(f: impl FnOnce()->u64 + std::panic::UnwindSafe) -> Option<u64> { std::panic::catch_unwind(f).ok() }','fn main() { std::panic::set_hook(Box::new(|_| {}));']
for i,row in enumerate(rows):
 if row['rust']:
  expected='None' if row['rust_expected'] is None else f'Some({row["rust_expected"]}u64)'
  rust.append(f'assert_eq!({row["rust"]}, {expected}, "case {i}: {row["call"]}");')
rust.append(f'println!("{sum(bool(r["rust"]) for r in rows)} pinned page/frame witnesses passed");\n}}\n');write(HERE/'src/main.rs','\n'.join(rust))
print(f'{len(rows)} numeric cases; {sum(bool(r["rust"]) for r in rows)} Rust witnesses')
