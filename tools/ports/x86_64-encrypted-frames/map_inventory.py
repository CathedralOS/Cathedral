#!/usr/bin/env python3
"""Exact frame anchors for explicit encryption-profile numeric components."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
UP=ROOT/'reference_code/rust-osdev/x86_64';PIN='cc35c876d3badb57df54a66e22f7768a52be95f2';target='source/libraries/x86_64/encrypted_frames.omg'
value=api.snapshot(UP,PIN,['src/structures/paging/frame.rs'],'https://github.com/rust-osdev/x86_64')
value['scope']='Explicit current-encryption-bit frame numeric algorithms and detached iterator state transitions. No frame identity, native representation, backing, mapping or Kani universal proof.'
selectors={26:'from_start',65:'from_pfn',88:'from_pfn',122:'containing',132:'from_start',139:'start_valid',160:'pfn',167:'range_count',174:'range_count',192:'arithmetic',199:'arithmetic',207:'arithmetic',214:'arithmetic',222:'difference',240:'range_count',246:'range_count',256:'range_bytes',265:'iterator_step',275:'iterator_step',298:'range_count',308:'iterator_step',317:'iterator_step',381:'range_count',387:'range_count',397:'range_bytes',406:'iterator_step',425:'iterator_step',448:'range_count',458:'iterator_step',476:'iterator_step'}
for path,file in value['files'].items():
 available={int(k.split(':',1)[0])for k in file['symbols']};assert set(selectors)<=available,set(selectors)-available
 file.update(disposition='translated',reason='Numeric expression/state-transition component with checked admission and explicit iterator failures; nominal/trait/proof infrastructure deliberately outside the component.',targets=[{'path':target,'anchor':'module encrypted_frames;'}])
 for key,row in file['symbols'].items():
  line=int(key.split(':',1)[0]);name=key.split(':',1)[1]
  if line in selectors:row.update(disposition='translated',reason='Explicit-profile numeric operation; checked NumberResult or detached cursor result replaces nominal wrapper and panic. Full cursor update order compared with actual pinned public iterators.',targets=[{'path':target,'anchor':'pub machine '+selectors[line]+'('}])
  else:row.update(disposition='omitted',reason='Nominal wrapper/field identity, formatting, unchecked construction, trait-associated plumbing, or Kani arbitrary/universal proof infrastructure remains outside this numeric component. Prior default-profile inventory retains its separate scope.')
text=json.dumps(value,indent=2,sort_keys=True)+'\n';out=ROOT/'source/libraries/x86_64/encrypted-frames-inventory.json'
if '--check'in sys.argv:assert out.read_text()==text
else:out.write_text(text)
print(api.check(value,UP,ROOT))
