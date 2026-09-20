#!/usr/bin/env python3
"""Retain captured-word translation expectations and actual pinned mapper calls."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
MASK=0x000ffffffffff000;MAX=2**64-1
rows=[]
for va in [0,0xdeadbeef,0xffff923456789abc]:
 for words in [(0,0,0,0),(128,0,0,0),(0x1081,0,0,0),(0x1001,0,0,0),(0x1001,128,0,0),(0x1001,0x2001,0,0),(0x1001,0x2001,0x3001,0),(0x1001,0x2001,0x3001,0x200),(0x1001,0x2001,0x3001,0xcafe0003),(0x1001,0x2001,0x3001,0xcafe0081),(0x1001,0x40001081,0,0),(0x1001,0x2001,0x40201081,0),(0x1001,MAX,0,0),(0x1001,0x2001,MAX,0)]:
  a,b,c,d=words;kind='mapped';word=d;size=4096
  if not a&1:kind='not_mapped'
  elif a&128:kind='root_huge'
  elif not b&1:kind='not_mapped'
  elif b&128:word=b;size=2**30
  elif not c&1:kind='not_mapped'
  elif c&128:word=c;size=2**21
  elif d==0:kind='not_mapped'
  row=dict(va=va,words=words,kind=kind)
  if kind=='mapped':row.update(frame=(word&MASK)&~(size-1),size=size,offset=va&(size-1),flags=word&~MASK)
  rows.append(row)
rows.append(dict(va=2**47,words=[0x1001,0x2001,0x3001,1],kind='invalid_virtual'))
prefix='// SPDX-License-Identifier: MIT OR Apache-2.0\nuse x86::translation;\nuse x86::translation::Translation;\n'
pieces=[]
for i,row in enumerate(rows):
 body=f'let result: Translation = translation::translate_words({row["va"]}, '+', '.join(map(str,row['words']))+');\n'
 if row['kind']=='mapped':
  body+='transition result { Translation::Mapped { frame, size, offset, flags } -> check(frame, size, offset, flags) _ -> (1) }\n'
  body+='state check(frame:u64,size:u64,offset:u64,flags:u64)->i32 { transition '+ ' && '.join(f'{field} == {row[field]}' for field in ['frame','size','offset','flags'])+' { true -> (0) _ -> (1) } }'
 else:body+='transition result in Translation::'+{'not_mapped':'NotMapped','root_huge':'InvalidRootHugePage','invalid_virtual':'InvalidVirtual'}[row['kind']]+' { true -> (0) _ -> (1) }'
 pieces.append(f'machine translation_case_{i}() -> i32 {{ {body} }}')
rust=r'''// SPDX-License-Identifier: MIT OR Apache-2.0
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{PageTable,PhysFrame,PageTableFlags};
use x86_64::structures::paging::mapper::{MappedPageTable,Translate,TranslateResult,PageTableFrameMapping};
struct Captures { p3: Box<PageTable>, p2: Box<PageTable>, p1: Box<PageTable> }
// The registry owns three stable, aligned, initialized allocations for the
// entire mapper lifetime. Every reachable non-leaf ID is one of these IDs;
// this test calls only immutable translate, never live mapping mutation.
unsafe impl PageTableFrameMapping for Captures {
 fn frame_to_pointer(&self,frame:PhysFrame)->*mut PageTable {
  let table=&**match frame.start_address().as_u64() { 0x1000=>&self.p3,0x2000=>&self.p2,0x3000=>&self.p1,_=>panic!("unregistered captured table") };
  table as *const PageTable as *mut PageTable
 }
}
fn store(table:&mut PageTable,index:usize,word:u64) {
 table[index].set_addr(PhysAddr::new(word&0x000ffffffffff000),PageTableFlags::from_bits_retain(word&0xfff0000000000fff));
}
fn translate(va:u64,words:[u64;4])->Option<(u64,u64,u64,u64)> {
 let va=VirtAddr::new(va);let mut root=Box::new(PageTable::new());
 let mut captures=Captures{p3:Box::new(PageTable::new()),p2:Box::new(PageTable::new()),p1:Box::new(PageTable::new())};
 store(&mut root,usize::from(va.p4_index()),words[0]);store(&mut captures.p3,usize::from(va.p3_index()),words[1]);
 store(&mut captures.p2,usize::from(va.p2_index()),words[2]);store(&mut captures.p1,usize::from(va.p1_index()),words[3]);
 // Registry and root remain alive and unique throughout the read-only call.
 let mapper=unsafe{MappedPageTable::new(&mut root,captures)};
 match mapper.translate(va){TranslateResult::Mapped{frame,offset,flags}=>Some((frame.start_address().as_u64(),frame.size(),offset,flags.bits())),TranslateResult::NotMapped=>None,TranslateResult::InvalidFrameAddress(_)=>panic!("unexpected invalid masked frame")}
}
fn main(){std::panic::set_hook(Box::new(|_|{}));
 let mut table=PageTable::new();assert!(table.is_empty());
 store(&mut table,0,0x12345003);store(&mut table,511,0x8000000000000000);
 assert_eq!(table[0].addr().as_u64()|table[0].flags().bits(),0x12345003);
 assert_eq!(table[511].flags().bits(),0x8000000000000000);assert!(!table.is_empty());
 table[0].set_unused();assert!(!table.is_empty());
 for entry in table.iter_mut(){entry.set_flags(PageTableFlags::from_bits_retain(u64::MAX));}
 assert_eq!(table.iter().filter(|entry|!entry.is_unused()).count(),512);
 table.zero();assert!(table.is_empty());assert!(table.iter().all(|entry|entry.is_unused()));
'''
for i,row in enumerate(rows):
 if row['kind']=='invalid_virtual':continue
 expr=f'translate({row["va"]}, ['+', '.join(map(str,row['words']))+'])'
 if row['kind']=='root_huge':rust+=f'assert!(std::panic::catch_unwind(||{expr}).is_err(),"case{i}");\n'
 else:
  e='None' if row['kind']=='not_mapped' else 'Some(('+','.join(str(row[key]) for key in ['frame','size','offset','flags'])+'))'
  rust+=f'assert_eq!({expr},{e},"case{i}");\n'
rust+='println!("42 actual pinned translations and complete512entry table operations passed");\n}\n'
def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale fixture',path)
 else:path.write_text(text)
write(HERE/'translations.json',json.dumps(rows,indent=2)+'\n')
write(HERE/'translations.omg',prefix+'\n'.join(pieces)+'\ndata Main{}\nmachine Main::main(&mut self){}\n')
write(HERE/'src/main.rs',rust)
print('43 Omega translation scenarios;42actual Rust mapper calls plus512entry operations')
