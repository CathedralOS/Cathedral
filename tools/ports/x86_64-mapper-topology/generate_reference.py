#!/usr/bin/env python3
"""Bind private numeric mirrors to exact pinned bodies; no live constructor calls."""
import argparse
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
PIN=ROOT/'reference_code/rust-osdev/x86_64/src/structures/paging/mapper'

def function(source,signature):
 start=source.index(signature);brace=source.index('{',start);depth=1;end=brace+1
 while depth:
  depth+=(source[end]=='{')-(source[end]=='}');end+=1
 return source[start:end]

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
 source=(PIN/'recursive_page_table.rs').read_text()
 parts=['''// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated from x86_64 cc35c876d3badb57df54a66e22f7768a52be95f2.
// Private coordinate bodies are exact copies, not public upstream API calls.
use x86_64::structures::paging::page::NotGiantPageSize;
''']
 for signature in ['fn p3_page<','fn p2_page<','fn p1_page(']:parts.append(function(source,signature))
 constructor=function(source,'pub fn new(table:')
 beginning=constructor.index('        let page =')
 end=constructor.index('        Ok(RecursivePageTable')
 body=constructor[beginning:end]
 assert 'VirtAddr::new(table as *const _ as u64)' in body and 'Ok(Cr3::read().0)' in body and 'table[recursive_index].frame()' in body
 body=body.replace('VirtAddr::new(table as *const _ as u64)','table_address').replace('Ok(Cr3::read().0)','Ok(observed_cr3)').replace('table[recursive_index].frame()','recursive_entry.frame()').replace('InvalidPageTable::','ObservedError::')
 parts.append('''// Observation mirror only: pointer acquisition -> supplied VirtAddr, CR3 read
// -> supplied PhysFrame, table indexing -> supplied entry, final Self -> index.
#[derive(Debug)] enum ObservedError { NotRecursive, NotActive }
fn observed_constructor(table_address:VirtAddr,observed_cr3:PhysFrame,recursive_entry:&PageTableEntry)->Result<PageTableIndex,ObservedError> {
'''+body+'        Ok(recursive_index)\n}')
 offset=(PIN/'offset_page_table.rs').read_text()
 assert 'let virt = self.offset + frame.start_address().as_u64();' in offset
 parts.append('''// Numeric expression extracted from private PhysOffset::frame_to_pointer.
// The following as_mut_ptr operation is deliberately absent.
fn offset_numeric(offset:VirtAddr,frame:PhysFrame)->VirtAddr {
    let virt = offset + frame.start_address().as_u64();
    virt
}
''')
 result='\n\n'.join(parts).rstrip()+'\n';path=HERE/'src/private_reference.rs'
 if args.check:assert path.read_text()==result,'private reference differs from current pinned extraction'
 else:path.write_text(result)
 print('Exact private coordinate bodies and explicit observation substitutions verified.')
if __name__=='__main__':main()
