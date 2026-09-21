#!/usr/bin/env python3
"""Profile-aware Translate overlay; unrelated mapper anchors remain pending."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
checkout=ROOT/'reference_code/rust-osdev/x86_64';pin='cc35c876d3badb57df54a66e22f7768a52be95f2'
path='src/structures/paging/mapper/recursive_page_table.rs'
data=shared.snapshot(checkout,pin,[path],'https://github.com/rust-osdev/x86_64')
target='source/libraries/x86_64/encrypted_recursive_constructor.omg'
file=data['files'][path];file.update(disposition='pending',reason='Explicit encryption profile extension to numeric recursive constructor observations; all other mapper components retain separate inventories.',targets=[{'path':target,'anchor':'pub machine observe_recursive'}])
for key,row in file['symbols'].items():
 if key=='57:new':row.update(disposition='translated',reason='Numeric constructor observations with explicit profile, preserving virtual geometry before strict current-bit frame admission and PRESENT/HUGE/frame comparison. No recursive object construction.',targets=[{'path':target,'anchor':'pub machine observe_recursive'}])
 else:row.update(disposition='pending',reason='Outside this extension; see existing route, topology and cleanup slices for their own scopes.')
out=ROOT/'source/libraries/x86_64/encrypted-constructor-inventory.json';text=json.dumps(data,indent=2)+'\n'
if '--check' in sys.argv:assert out.read_text()==text,'stale encrypted recursive translation inventory'
else:out.write_text(text)
print(shared.check(data,checkout))
