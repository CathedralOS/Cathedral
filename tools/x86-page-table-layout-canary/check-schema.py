#!/usr/bin/env python3
"""Bind the local projection fixture to the actual canonical PTE field schema."""
import re
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
original=(ROOT/'source/drivers/facts/x86_page_table_entry.omg').read_text()
fixture=(HERE/'main.omg').read_text()
fields=lambda text,name:re.search(r'data '+name+r' \{([^}]+)\}',text)[1].split()
assert fields(original,'X86PageTableEntry')==fields(fixture,'LocalPageTableEntry')
policy=(ROOT/'source/drivers/facts/x86_page_table_layout.omg').read_text()
rows=re.findall(r'key: schema.fields\[(\d+)\].key,\s*placement: FieldPlan::Bits \{\s*container: (\d+),\s*container_width: (\d+),\s*destination_lsb: (\d+),\s*source_lsb: (\d+),\s*width: (\d+),',policy)
expected=[(i,0,64,offset,0,width) for i,(offset,width) in enumerate([(i,1) for i in range(9)]+[(9,3),(12,40),(52,7),(59,4),(63,1)])]
assert [tuple(map(int,row)) for row in rows]==expected
for text in ['entry_count: 14','size_fixed: 8','size_is_dynamic: false','align: 8']:assert text in policy
print('Canonical14field schema and all64requested bit positions agree; not native ABI evidence.')
