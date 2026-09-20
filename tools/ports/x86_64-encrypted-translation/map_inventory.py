#!/usr/bin/env python3
"""Profile-aware Translate overlay; unrelated mapper anchors remain pending."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
checkout=ROOT/'reference_code/rust-osdev/x86_64';pin='cc35c876d3badb57df54a66e22f7768a52be95f2'
path='src/structures/paging/mapper/mapped_page_table.rs'
data=shared.snapshot(checkout,pin,[path],'https://github.com/rust-osdev/x86_64')
target='source/libraries/x86_64/encrypted_translation.omg'
file=data['files'][path];file.update(disposition='pending',reason='Explicit encryption profile extension to the existing captured generic translation; all other mapper components retain separate inventories.',targets=[{'path':target,'anchor':'pub machine translate_words'}])
for key,row in file['symbols'].items():
 if key=='487:translate':row.update(disposition='translated',reason='Numeric selected-word translation with explicit encryption profile. Child snapshot identities checked separately by walk_path; no live mapper or pointer conversion.',targets=[{'path':target,'anchor':'pub machine translate_words'},{'path':target,'anchor':'pub machine walk_path'}])
 else:row.update(disposition='pending',reason='Outside this extension; see existing route, topology and cleanup slices for their own scopes.')
out=ROOT/'source/libraries/x86_64/encrypted-translation-inventory.json';text=json.dumps(data,indent=2)+'\n'
if '--check' in sys.argv:assert out.read_text()==text,'stale encrypted translation inventory'
else:out.write_text(text)
print(shared.check(data,checkout))
