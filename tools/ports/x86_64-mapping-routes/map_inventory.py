#!/usr/bin/env python3
"""Audit pure captured-route coverage separately from live mapper ownership."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
doc=json.loads((ROOT/'source/libraries/x86_64/mapping-plans-inventory.json').read_text())
target='source/libraries/x86_64/mapping_routes.omg'
methods={'map_to_with_table_flags','unmap','update_flags','translate_page','set_flags_p4_entry','set_flags_p3_entry','set_flags_p2_entry'}
for path,file in doc['files'].items():
 for key,row in file['symbols'].items():
  _,name=key.split(':',1)
  if path.endswith('/mapped_page_table.rs') and name in methods:
   row.update(disposition='translated',reason='Complete pure route order over selected initialized snapshots, allocation observations and retained edits. Live hierarchy references, mapping/allocator custody and invalidation settlement remain explicit external interfaces.',targets=[dict(path=target,anchor='pub machine plan(')])
  elif path.endswith('/mapped_page_table.rs') and name in {'next_table','next_table_mut','PageTableWalkError','PageTableCreateError','from'}:
   row.update(disposition='translated',reason='PRESENT-before-HUGE traversal and error propagation are semantic outcomes in the captured route. Numeric table IDs are compared before a supplied existing child snapshot is used; no pointer is constructed.',targets=[dict(path=target,anchor='pub data Outcome [copy]')])
  elif path.endswith('/mapped_page_table.rs') and name=='translate':
   row.update(disposition='translated',reason='Previously tested complete size-discovering captured translation; distinct from size-specific route translation.',targets=[dict(path='source/libraries/x86_64/translation.omg',anchor='pub machine translate_words(')])
  elif path.endswith('/mod.rs') and name in methods:
   row.update(disposition='translated',reason='Pure numeric method behavior is represented by captured route requests; Rust mutable mapper/allocator traits are not authority types.',targets=[dict(path=target,anchor='pub data Request [copy]')])
  elif path.endswith('/mod.rs') and name in {'MapToError','UnmapError','FlagUpdateError','TranslateError'}:
   row.update(disposition='translated',reason='Operation-specific semantic cases and payloads are retained by route Outcome wrapping the tested ChildPlan/LeafPlan; invalid raw inputs add explicit rejections.',targets=[dict(path=target,anchor='pub data Outcome [copy]')])
 file.update(disposition='pending',reason='Pure captured mapper routes are translated; cleanup, convenience wrappers and live authority/flush surfaces remain separate pending work.',targets=[dict(path=target,anchor='module mapping_routes;')])
text=json.dumps(doc,indent=2)+'\n';path=ROOT/'source/libraries/x86_64/mapping-routes-inventory.json'
if '--check' in sys.argv:assert path.read_text()==text,'stale route manifest'
else:path.write_text(text)
