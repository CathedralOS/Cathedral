#!/usr/bin/env python3
"""Retain whole-range cleanup as pending while mapping its tested branch kernel."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
doc=json.loads((ROOT/'source/libraries/x86_64/mapping-plans-inventory.json').read_text())
doc['files']={p:f for p,f in doc['files'].items()if p.endswith('/mapped_page_table.rs')}
doc['upstream']['roots']=list(doc['files'])
target='source/libraries/x86_64/cleanup_branch.omg'
for f in doc['files'].values():
 f.update(disposition='pending',reason='Only the single-page cleanup branch is translated here; arbitrary-range traversal, live custody and other mapper operations are outside this narrow slice.',targets=[dict(path=target,anchor='module cleanup_branch;')])
 for key,row in f['symbols'].items():
  row.update(disposition='pending',reason='Outside the single-page cleanup branch slice; see the separate captured-route and table inventories.');row.pop('targets',None)
  if key.split(':',1)[1]in ['clean_up','clean_up_addr_range']:
   row.update(reason='Single selected-page branch translated and tested, including whole-table emptiness observations and bottom-up detach/retirement order. Complete arbitrary-range orchestration and custody integration remain pending.',targets=[dict(path=target,anchor='pub machine plan('),dict(path=target,anchor='pub machine has_other_entries(')])
text=json.dumps(doc,indent=2)+'\n';path=ROOT/'source/libraries/x86_64/cleanup-branch-inventory.json'
if '--check'in sys.argv:assert path.read_text()==text,'stale cleanup branch manifest'
else:path.write_text(text)
