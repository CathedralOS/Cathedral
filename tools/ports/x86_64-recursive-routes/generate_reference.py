#!/usr/bin/env python3
"""Copy recursive decision bodies; replace raw topology access with safe borrowed snapshots."""
from pathlib import Path
import importlib.util,re,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];UP=ROOT/'reference_code/rust-osdev/x86_64';PIN='cc35c876d3badb57df54a66e22f7768a52be95f2'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
i.snapshot(UP,PIN,['src/structures/paging/mapper/recursive_page_table.rs'],'https://github.com/rust-osdev/x86_64')
s=(UP/'src/structures/paging/mapper/recursive_page_table.rs').read_text()
def body(text,start):
 n=text.index(start);a=text.index('{',n);depth=1;b=a+1
 while depth:
  depth+=(text[b]=='{')-(text[b]=='}');b+=1
 return text[a+1:b-1]
header='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated pinned branch/mutation bodies with topology accesses explicitly replaced.
// No RecursivePageTable instance, raw pointer, live root or instruction exists here.
#![allow(unused_variables,unused_unsafe,non_snake_case)]
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page,PageSize,Size4KiB,Size2MiB,Size1GiB,PhysFrame,PageTableFlags,page::AddressNotAligned,page_table::FrameError,mapper::{MapToError,UnmapError,FlagUpdateError,TranslateError,MapperFlush,MapperFlushAll,TranslateResult,MappedFrame}};
use crate::{ObservedTable as PageTable,ObservedEntry as PageTableEntry,AllocationObservations as FrameAllocator};
'''
inner=body(s,"        fn inner<'b, A, S: PageSize>(")
old='''            let page_table_ptr = next_table_page.start_address().as_mut_ptr();
            let page_table: &mut PageTable = unsafe { &mut *(page_table_ptr) };'''
assert inner.count(old)==1;inner=inner.replace(old,'            let page_table: &mut PageTable = next_table_page;')
header+='''fn create_next_table<'b,A,S:PageSize>(entry:&mut PageTableEntry,next_table_page:&'b mut PageTable,insert_flags:PageTableFlags,allocator:&mut A)->Result<&'b mut PageTable,MapToError<S>> where A:FrameAllocator<Size4KiB>+?Sized {'''+inner+'}\n'
for size in ['Size1GiB','Size2MiB','Size4KiB']:
 start=s.index('impl Mapper<'+size+'>');end=s.index('\nimpl ',start+1);block=s[start:end]
 for method in ['map_to_with_table_flags','unmap','update_flags','set_flags_p4_entry','set_flags_p3_entry','set_flags_p2_entry','translate_page']:
  b=body(block,'fn '+method)
  b=b.replace('        let p4 = &mut self.p4;','').replace('        let p4 = &self.p4;','')
  for level in [3,2,1]:
   b=b.replace(f'let p{level}_page = p{level}_page(page, self.recursive_index);',f'let p{level}_page = table{level};')
   b=b.replace(f'let p{level} = unsafe {{ &mut *(p{level}_ptr(page, self.recursive_index)) }};',f'let p{level} = table{level};')
   b=b.replace(f'let p{level} = unsafe {{ &*(p{level}_ptr(page, self.recursive_index)) }};',f'let p{level} = table{level};')
  b=b.replace('Self::create_next_table(','create_next_table(')
  assert 'self.' not in b and '_ptr(' not in b,(size,method)
  generics='<A>' if method=='map_to_with_table_flags' else ''
  args=f'p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<{size}>'
  if method=='map_to_with_table_flags':args+=f',frame:PhysFrame<{size}>,flags:PageTableFlags,parent_table_flags:PageTableFlags,allocator:&mut A';ret=f'Result<MapperFlush<{size}>,MapToError<{size}>> where A:FrameAllocator<Size4KiB>+?Sized'
  elif method=='unmap':ret=f'Result<(PhysFrame<{size}>,MapperFlush<{size}>),UnmapError>'
  elif method=='translate_page':ret=f'Result<PhysFrame<{size}>,TranslateError>'
  else:
   args+=',flags:PageTableFlags';ret=f'Result<MapperFlush<{size}>,FlagUpdateError>' if method=='update_flags' else 'Result<MapperFlushAll,FlagUpdateError>'
  header+=f'pub fn {method}_{size}{generics}({args})->{ret} {{'+b+'}\n'
b=body(s[s.index('impl Translate for RecursivePageTable'):],'fn translate(')
b=b.replace('        let p4 = &self.p4;','').replace('let page = Page::containing_address(addr);','let page: Page<Size4KiB> = Page::containing_address(addr);')
for level in [3,2,1]:b=b.replace(f'let p{level} = unsafe {{ &*(p{level}_ptr(page, self.recursive_index)) }};',f'let p{level} = table{level};')
assert 'self.' not in b and '_ptr(' not in b
header+='pub fn translate(p4:&PageTable,table3:&PageTable,table2:&PageTable,table1:&PageTable,addr:VirtAddr)->TranslateResult {'+b+'}\n'
header=header.replace('use crate::structures::paging::PageTableFlags as Flags;', 'use x86_64::structures::paging::PageTableFlags as Flags;')
assert 'as_mut_ptr' not in header and 'asm!' not in header
out=HERE/'src/pinned.rs'
if '--check' in sys.argv:
 if not out.exists() or out.read_text()!=header:raise SystemExit('recursive pinned mirror drift')
else:out.write_text(header)
print('Extracted21 sized method bodies, child body and generic translation; raw accesses replaced with borrowed snapshots')
