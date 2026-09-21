#!/usr/bin/env python3
"""Retain exact decision bodies, substituting only owned snapshot access."""
from pathlib import Path
import importlib.util,re,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];UP=ROOT/'reference_code/rust-osdev/x86_64';PIN='cc35c876d3badb57df54a66e22f7768a52be95f2'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
files=['src/structures/paging/mapper/mapped_page_table.rs','src/structures/paging/mapper/recursive_page_table.rs','src/structures/paging/page_table.rs','src/structures/mem_encrypt.rs','src/addr.rs'];i.snapshot(UP,PIN,files,'https://github.com/rust-osdev/x86_64')
def body(text,start):
 n=text.index(start);a=text.index('{',n);depth=1;b=a+1
 while depth:depth+=(text[b]=='{')-(text[b]=='}');b+=1
 return text[a+1:b-1]
header='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated pinned branch bodies; borrowed snapshots replace all raw topology access.
#![allow(unused_variables,unused_unsafe,unused_imports,non_snake_case)]
use x86_64::structures::paging::{Page,PageSize,Size4KiB,Size2MiB,Size1GiB,PhysFrame,PageTableFlags,page::AddressNotAligned,page_table::FrameError,mapper::{MapToError,UnmapError,FlagUpdateError,TranslateError,MapperFlush,MapperFlushAll}};
use crate::{ObservedTable as PageTable,ObservedEntry as PageTableEntry,AllocationObservations as FrameAllocator};
'''
outputs={}
for mode in ['mapped','recursive']:
 s=(UP/f'src/structures/paging/mapper/{mode}_page_table.rs').read_text();out=header
 if mode=='recursive':
  inner=body(s,"        fn inner<'b, A, S: PageSize>(")
  old='''            let page_table_ptr = next_table_page.start_address().as_mut_ptr();
            let page_table: &mut PageTable = unsafe { &mut *(page_table_ptr) };'''
  assert inner.count(old)==1;inner=inner.replace(old,'            let page_table: &mut PageTable = next_table_page;')
  out+='''fn create_next_table<'b,A,S:PageSize>(entry:&mut PageTableEntry,next_table_page:&'b mut PageTable,insert_flags:PageTableFlags,allocator:&mut A)->Result<&'b mut PageTable,MapToError<S>> where A:FrameAllocator<Size4KiB>+?Sized {'''+inner+'}\n'
 else:
  out+=s[s.index('#[derive(Debug)]\nenum PageTableWalkError'):s.index('/// Provides a virtual address mapping')]
  for name,mut in [('next_table',''),('next_table_mut','mut ')]:
   b=body(s,f"fn {name}<'b>(")
   start=b.index('        let page_table_ptr');end=b.index('\n\n        Ok(page_table)',start)
   b=b[:start]+'        entry.frame()?;\n        let page_table = table;'+b[end:]
   out+=f"fn {name}<'b>(table:&'b {mut}PageTable,entry:&{mut}PageTableEntry)->Result<&'b {mut}PageTable,PageTableWalkError>{{"+b+'}\n'
  b=body(s,"fn create_next_table<'b, A>(").replace('self.next_table_mut(entry)','next_table_mut(table,entry)')
  out+="fn create_next_table<'b,A>(table:&'b mut PageTable,entry:&mut PageTableEntry,insert_flags:PageTableFlags,allocator:&mut A)->Result<&'b mut PageTable,PageTableCreateError> where A:FrameAllocator<Size4KiB>+?Sized {"+b+'}\n'
 for size in ['Size1GiB','Size2MiB','Size4KiB']:
  key='impl Mapper<'+size+'>' if mode=='recursive' else 'impl<P: PageTableFrameMapping> Mapper<'+size+'>'
  start=s.index(key);end=s.index('\nimpl ',start+1);block=s[start:end]
  for method in ['map_to_with_table_flags','unmap','update_flags','set_flags_p4_entry','set_flags_p3_entry','set_flags_p2_entry','translate_page']:
   b=body(block,'fn '+method)
   b=re.sub(r'        let p4 = &(?:mut )?self\.(?:p4|level_4_table);','',b)
   if mode=='recursive':
    for level in [3,2,1]:
     b=b.replace(f'let p{level}_page = p{level}_page(page, self.recursive_index);',f'let p{level}_page = table{level};')
     for mutable in ['mut ','']:b=b.replace(f'let p{level} = unsafe {{ &{mutable}*(p{level}_ptr(page, self.recursive_index)) }};',f'let p{level} = table{level};')
    b=b.replace('Self::create_next_table(','create_next_table(')
   else:
    b=re.sub(r'let p([123]) = self\s*\.page_table_walker\s*\.(create_next_table|next_table_mut|next_table)\(',lambda m:f'let p{m[1]} = {m[2]}(table{m[1]}, ',b)
   assert 'self.' not in b and '_ptr(' not in b,(mode,size,method)
   gen='<A>' if method=='map_to_with_table_flags' else '';args=f'p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<{size}>'
   if method=='map_to_with_table_flags':args+=f',frame:PhysFrame<{size}>,flags:PageTableFlags,parent_table_flags:PageTableFlags,allocator:&mut A';ret=f'Result<MapperFlush<{size}>,MapToError<{size}>> where A:FrameAllocator<Size4KiB>+?Sized'
   elif method=='unmap':ret=f'Result<(PhysFrame<{size}>,MapperFlush<{size}>),UnmapError>'
   elif method=='translate_page':ret=f'Result<PhysFrame<{size}>,TranslateError>'
   else:args+=',flags:PageTableFlags';ret=f'Result<MapperFlush<{size}>,FlagUpdateError>' if method=='update_flags' else 'Result<MapperFlushAll,FlagUpdateError>'
   out+=f'pub fn {method}_{size}{gen}({args})->{ret}{{'+b+'}\n'
 out=out.replace('use crate::structures::paging::PageTableFlags as Flags;','use x86_64::structures::paging::PageTableFlags as Flags;')
 assert 'as_mut_ptr' not in out and 'asm!' not in out
 outputs[HERE/'src'/f'{mode}.rs']='\n'.join(line.rstrip() for line in out.splitlines())+'\n'
for p,s in outputs.items():
 if '--check' in sys.argv:
  if not p.exists() or p.read_text()!=s:raise SystemExit('reference drift '+str(p))
 else:p.write_text(s)
print('PASS extracted42 method bodies plus child/walker/error branches')
