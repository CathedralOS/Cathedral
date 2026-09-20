#!/usr/bin/env python3
"""Extract exact pure pinned bodies; omit assembly and CPU observations explicitly."""
from pathlib import Path
import hashlib,importlib.util,json,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];PIN='cc35c876d3badb57df54a66e22f7768a52be95f2'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
i.snapshot(ROOT/'reference_code/rust-osdev/x86_64',PIN,['src/instructions/tlb.rs'],'https://github.com/rust-osdev/x86_64')
s=(ROOT/'reference_code/rust-osdev/x86_64/src/instructions/tlb.rs').read_text()
command=s[s.index('#[derive(Debug)]\npub enum InvPcidCommand'):s.index('// TODO: Remove this')]
p=s[s.index('/// The INVPCID descriptor'):s.index('/// Invalidate the given address in the TLB using the `invpcid`')]
p=p.replace('struct InvpcidDescriptor','pub struct InvpcidDescriptor')
body=s[s.index('    let mut desc = InvpcidDescriptor'):s.index('    unsafe {\n        asm!("invpcid')]
flush=s[s.index('    let mut rax = 0;'):s.rindex('    unsafe {\n        asm!(')]
chunk=s[s.index('                // Calculate out how many pages'):s.index('                unsafe {\n                    flush_broadcast(')]
chunk=chunk.replace('Page::<S>::steps_between_impl','<Page<S> as Step>::steps_between').replace('Page::steps_between_impl','<Page<S> as Step>::steps_between').replace('self.invlpgb.invlpgb_count_max','count_max')
source='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated exact pure fragments from pinned instructions/tlb.rs. No instruction execution.
use bit_field::BitField;
use core::{cmp,fmt};
use std::iter::Step;
use x86_64::{VirtAddr,structures::paging::{Page,PageSize,Size2MiB,page::NotGiantPageSize}};
'''+command+p+'\npub fn prepare_invpcid(command: InvPcidCommand)->(u64,InvpcidDescriptor) {\n'+body+'    (kind,desc)\n}\n'+'''pub fn encode_broadcast<S:NotGiantPageSize>(va_and_count:Option<(Page<S>,u16)>,pcid:Option<Pcid>,asid:Option<u16>,include_global:bool,final_translation_only:bool,include_nested_translations:bool)->(u64,u32,u32) {
'''+flush+'    (rax,ecx,edx)\n}\n'+'''pub fn pinned_chunk<S:NotGiantPageSize>(start:Page<S>,end:Page<S>,count_max:u16)->Option<(u16,u16,u64)> {
    let pages=Page::range(start,end);
    if pages.is_empty(){return None;}
'''+chunk+'''    let inc_count=cmp::max(count,1);
    let next=<Page<S> as Step>::forward_checked(pages.start,usize::from(inc_count)).unwrap();
    Some((count,inc_count,next.start_address().as_u64()))
}
'''
source+='pub fn descriptor_offsets()->(usize,usize){(core::mem::offset_of!(InvpcidDescriptor,pcid),core::mem::offset_of!(InvpcidDescriptor,address))}\n'
out=HERE/'src/pinned.rs'
if '--check' in sys.argv:
 if not out.exists() or out.read_text()!=source:raise SystemExit('pinned body mirror drift')
else:out.write_text(source)
print('Pinned PCID/INVPCID descriptor+match/broadcast composition/range chunk bodies extracted; hardware tails omitted')
