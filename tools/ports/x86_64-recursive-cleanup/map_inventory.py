#!/usr/bin/env python3
"""Bind all pinned recursive mapper anchors; map only detached cleanup here."""
import json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
upstream='src/structures/paging/mapper/recursive_page_table.rs'
result=subprocess.check_output([sys.executable,str(ROOT/'tools/ports/inventory.py'),'snapshot','--checkout',str(ROOT/'reference_code/rust-osdev/x86_64'),'--revision','cc35c876d3badb57df54a66e22f7768a52be95f2','--url','https://github.com/rust-osdev/x86_64',upstream],text=True)
data=json.loads(result);data['slice']='recursive cleanup detached numeric orchestration only; live mutation/deallocation interfaces are not ported'
for file in data['files'].values():
 file['disposition']='pending';file['reason']='Full file audited as context; only bounded detached cleanup family is covered by this supplement.'
 for key,row in file['symbols'].items():
  row['disposition']='pending';row['reason']='Outside recursive cleanup supplement; see separate topology, routes and translation slices. Live pointers, hierarchy custody and instructions remain outside numeric ports.'
  if key.split(':',1)[1]in ['clean_up','clean_up_addr_range']:
   row['disposition']='translated';row['targets']=[{'path':'source/libraries/x86_64/recursive_cleanup.omg','anchor':'pub machine '+name+'('}for name in ['begin','resume','step']];row['reason']='Modified detached begin/resume/step orchestration with explicit recursive-slot exclusion, finite budget, and ordinary cleanup plans. Exact private clean_up mirror validates owned fixture outcomes. No live API, raw recursive reference, actual entry write or frame retirement.'
path=ROOT/'source/libraries/x86_64/recursive-cleanup-inventory.json';text=json.dumps(data,indent=2,sort_keys=True)+'\n'
if '--check'in sys.argv:assert path.read_text()==text,'stale recursive cleanup inventory'
else:path.write_text(text)
