#!/usr/bin/env python3
"""Overlay completed borrowed-table and captured-translation operations only."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
p=ROOT/'source/libraries/x86_64/tables-inventory.json';doc=json.loads(p.read_text())
base=json.loads((ROOT/'source/libraries/x86_64/page-entries-inventory.json').read_text())
table='source/libraries/x86_64/page_tables.omg';translation='source/libraries/x86_64/translation.omg'
for path,file in doc['files'].items():
 for key,row in file['symbols'].items():
  number,name=key.split(':',1);n=int(number);target=None;anchor=None
  if path.endswith('/page_table.rs'):
   original=base['files'][path]['symbols'][key]
   if original['disposition']!='pending':row.update(original);continue
   if name=='PageTable':target='source/core/x86_page_table.omg';anchor='pub data X86PageTablePageCandidate {';reason='Canonical core candidate retained; pure helpers borrow its initialized array shape without creating a competing table carrier.'
   elif name=='ENTRY_COUNT':target=table;anchor='pub const ENTRY_COUNT:';reason='Fixed512entry geometry retained.'
   elif name in {'Output','fmt','default'}:row.update(disposition='omitted',reason='Rust presentation/index-trait/default scaffolding replaced by explicit array operations and new_table.');row.pop('targets',None);continue
   else:
    mapping={'new':'new_table','EMPTY':'new_table','zero':'zero','iter':'read_word','iter_mut':'write_word','is_empty':'is_empty','index':'read_word','index_mut':'write_word'}
    target=table;anchor='pub machine '+mapping[name]+'(';reason='Initialized array operation retained; bounded caller-owned indices replace unsafe pointer iteration and trait indexing. No physical reference is fabricated.'
  elif path.endswith('/mapped_page_table.rs') and key=='487:translate':
   target=translation;anchor='pub machine translate_words(';reason='Pure numeric branching retained from actual mapper translate; borrowed captured-array wrapper checks selected coordinates and supplied link identities. Physical dereferences become explicit initialized snapshots. Root huge-page panic becomes a semantic error case.'
  elif path.endswith('/mod.rs') and name in {'Translate','translate','translate_addr','TranslateResult','MappedFrame','start_address','size'}:
   target=translation;anchor='pub machine translated_address(' if name=='translate_addr' else 'pub data Translation {' if name in {'TranslateResult','MappedFrame','start_address','size'} else 'pub machine translate_words('
   reason='Pure translation result/operation retained as explicit semantic cases and numeric fields. Leaf flags preserve upstream behavior and are not effective hardware permissions.'
  else:
   row.update(disposition='pending',reason='Mapped/recursive hierarchy mutation, ownership adapters and related interface/error/flush/cleanup surfaces remain separate engineering or later authority design; this slice makes no completion/blocker claim.');row.pop('targets',None);continue
  row.update(disposition='translated',reason=reason,targets=[dict(path=target,anchor=anchor)])
 file.update(disposition='pending' if any(r['disposition']=='pending' for r in file['symbols'].values()) else 'translated',reason='Bounded numeric source overlay; remaining mapper interfaces and operations explicitly pending.',targets=[dict(path=table if path.endswith('/page_table.rs') else translation,anchor='module page_tables;' if path.endswith('/page_table.rs') else 'module translation;')])
text=json.dumps(doc,indent=2)+'\n'
if '--check' in sys.argv:assert p.read_text()==text,'stale reviewed table mapping'
else:p.write_text(text)
