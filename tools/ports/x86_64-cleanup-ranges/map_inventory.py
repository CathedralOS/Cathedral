#!/usr/bin/env python3
"""Map complete pure cleanup orchestration while retaining live interfaces."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
doc=json.loads((ROOT/'source/libraries/x86_64/cleanup-branch-inventory.json').read_text())
for path,file in doc['files'].items():
 for key,row in file['symbols'].items():
  if key.split(':',1)[1]in ['clean_up','clean_up_addr_range']:
   row.update(disposition='translated',reason='Bounded resumable pure range orchestration and tested deepest-first branch cleanup. Initialized snapshots are caller-supplied numeric observations; actual parent mutation, deallocation, custody and invalidation remain external integration.',targets=[dict(path='source/libraries/x86_64/cleanup_ranges.omg',anchor='pub machine step('),dict(path='source/libraries/x86_64/cleanup_branch.omg',anchor='pub machine plan(')])
 file.update(disposition='pending',reason='Pure cleanup algorithms translated; other mapper methods have separate slice inventories. Live ownership, mutable references and invalidation interfaces remain outside numeric proposals.')
text=json.dumps(doc,indent=2)+'\n';path=ROOT/'source/libraries/x86_64/cleanup-ranges-inventory.json'
if '--check'in sys.argv:assert path.read_text()==text,'stale cleanup range manifest'
else:path.write_text(text)
