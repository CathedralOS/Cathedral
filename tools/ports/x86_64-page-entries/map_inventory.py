#!/usr/bin/env python3
"""Review bounded PTE/index slice; detached table operations remain pending."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
p=ROOT/'source/libraries/x86_64/page-entries-inventory.json';doc=json.loads(p.read_text())
target='source/libraries/x86_64/page_entries.omg';facts='source/drivers/facts/x86_page_table_entry.omg'
for path,file in doc['files'].items():
 for key,row in file['symbols'].items():
  n,name=key.split(':');n=int(n);dest=target;anchor=None;status='translated';reason='Checked numeric operation over canonical Cathedral PTE schema; no memory access or hardware authority.'
  if 202<=n<=312:
   status='pending';reason='Detached full-table storage/iteration remains subsequent engineering; existing core page candidate remains canonical.'
  elif name in {'fmt','from','into_u64','tests'} or name=='default':
   status='omitted';reason='Rust nominal/formatting/conversion scaffolding omitted; raw numeric results and explicit constructors preserve the retained operations.'
  elif n in {28,115}:
   status='omitted';reason='Ambient atomic memory-encryption configuration is outside this explicit default-mask slice; addresses.omg separately supports explicit mask inputs.'
  elif name in {'PageTableEntry','PageTableFlags'}:dest=facts;anchor='pub data X86PageTableEntry {';reason='Reuses canonical field schema, including software/protection-key fields; no duplicate wrapper or bitflags type.'
  elif name.isupper():anchor='pub machine decode(';reason='Pinned flag bit is represented in the existing canonical field(s), decoded and encoded without loss; no redundant flag wrapper. Single-bit fixtures cover every position.'
  elif name=='FrameError':anchor='pub data FrameResult {'
  elif name in {'new','set_unused'} and n<100:anchor='pub machine decode(';reason='Zero construction/reset represented by decode(0) and ordinary owned-value replacement; no interior mutable alias.'
  elif name=='physical_address_mask':anchor='pub const ADDRESS_MASK:'
  elif name in {'PageTableIndex','PageOffset','PageTableLevel','One'}:
   anchor='pub machine '+{'PageTableIndex':'index_checked','PageOffset':'offset_checked','PageTableLevel':'next_level','One':'next_level'}[name]+'(';reason='Nominal wrapper/level becomes checked explicit numeric input; no unchecked foreign integer establishes semantic membership.'
  elif name=='new':anchor='pub machine '+('index_checked' if n==328 else 'offset_checked')+'('
  elif name=='new_truncate':anchor='pub machine '+('index_truncate' if n==335 else 'offset_truncate')+'('
  elif name=='page_table_index_step_overflowing':dest='tools/ports/x86_64-page-entries/generate.py';anchor='index_overflowing(';reason='All four pinned overflowing-step assertions execute against actual Rust and translated Omega bodies, plus extreme-count cases.'
  else:
   mapping={'is_unused':'is_unused','flags':'flags','addr':'address','frame':'frame','set_addr':'set_address','set_frame':'set_frame','set_flags':'set_flags','steps_between':'index_distance','forward_checked':'index_step','backward_checked':'index_step','forward_overflowing':'index_overflowing','backward_overflowing':'index_overflowing','next_lower_level':'next_level','next_higher_level':'next_level','table_address_space_alignment':'level_alignment','entry_address_space_alignment':'level_alignment'}
   if name not in mapping:raise ValueError(key)
   anchor='pub machine '+mapping[name]+'('
  row.update(disposition=status,reason=reason)
  if anchor:row['targets']=[dict(path=dest,anchor=anchor)]
  else:row.pop('targets',None)
 file.update(disposition='pending',reason='PTE word/index/level slice implemented; complete detached table algorithms remain engineering.',targets=[dict(path=target,anchor='module page_entries;')])
text=json.dumps(doc,indent=2)+'\n'
if '--check' in sys.argv:assert p.read_text()==text,'stale reviewed source mapping'
else:p.write_text(text)
