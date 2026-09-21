#!/usr/bin/env python3
"""Extract constructor checks, replacing pointer/CR3/table inputs with values."""
import importlib.util,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('extract',ROOT/'tools/ports/x86_64-mapper-topology/generate_reference.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
source=(ROOT/'reference_code/rust-osdev/x86_64/src/structures/paging/mapper/recursive_page_table.rs').read_text()
body=mod.function(source,'pub fn new(table:');body=body[body.index('        let page ='):body.index('        Ok(RecursivePageTable')]
for old,new in [('VirtAddr::new(table as *const _ as u64)','table_address'),('Ok(Cr3::read().0)','Ok(observed_frame(observed))'),('table[recursive_index].frame()','recursive_entry.frame()'),('InvalidPageTable::','ObservedError::')]:
 assert old in body,old;body=body.replace(old,new)
text='// SPDX-License-Identifier: MIT OR Apache-2.0\n// Exact constructor checks with explicit numeric observation substitutions.\n#[derive(Debug)] enum ObservedError {NotRecursive,NotActive}\nfn observed_constructor(table_address:VirtAddr,observed:u64,recursive_entry:&PageTableEntry)->Result<PageTableIndex,ObservedError> {\n'+body+'    Ok(recursive_index)\n}\n'
path=HERE/'src/pinned.rs'
if '--check' in sys.argv:assert path.read_text()==text
else:path.write_text(text)
print('Pinned constructor observation substitutions verified')
