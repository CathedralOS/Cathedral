#!/usr/bin/env python3
"""Extract the complete recursive Translate body with explicit safe topology substitution."""
import sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
source=(ROOT/'reference_code/rust-osdev/x86_64/src/structures/paging/mapper/recursive_page_table.rs').read_text()
begin=source.index('fn translate(',source.index('impl Translate for RecursivePageTable'))
a=source.index('{',begin);b=a+1;depth=1
while depth:depth+=(source[b]=='{')-(source[b]=='}');b+=1
body=source[a+1:b-1]
replacements={'        let p4 = &self.p4;':'','let page = Page::containing_address(addr);':'let page: Page<Size4KiB> = Page::containing_address(addr);'}
for level in [3,2,1]:replacements[f'let p{level} = unsafe {{ &*(p{level}_ptr(page, self.recursive_index)) }};']=f'let p{level} = table{level};'
for old,new in replacements.items():assert body.count(old)==1,old;body=body.replace(old,new)
assert 'self.' not in body and 'unsafe' not in body and '_ptr(' not in body
text='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Exact recursive translate body at cc35c876d3badb57df54a66e22f7768a52be95f2,
// replacing only topology resolution with disjoint initialized table borrows.
#![allow(unused_variables)]
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page,Size4KiB,PageTable,PageTableFlags,PhysFrame,page::AddressNotAligned,mapper::{TranslateResult,MappedFrame}};
pub fn translate(p4:&PageTable,table3:&PageTable,table2:&PageTable,table1:&PageTable,addr:VirtAddr)->TranslateResult {'''+body+'}\n'
path=HERE/'src/pinned.rs'
if '--check' in sys.argv:assert path.read_text()==text,'pinned body drift'
else:path.write_text(text)
print('Exact recursive translation body; only topology borrows and page type inference substituted')
