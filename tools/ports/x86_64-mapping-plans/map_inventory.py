#!/usr/bin/env python3
"""Name pure extracted components without declaring full mapper calls complete."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
p=ROOT/'source/libraries/x86_64/mapping-plans-inventory.json';doc=json.loads(p.read_text())
target='source/libraries/x86_64/mapping_plans.omg'
for path,file in doc['files'].items():
 for key,row in file['symbols'].items():
  n,name=key.split(':',1);n=int(n);row.update(disposition='pending',reason='Full mapper orchestration, hierarchy access/ownership, cleanup and invalidation interfaces remain subsequent work.');row.pop('targets',None)
  if path.endswith('/mapped_page_table.rs') and name=='create_next_table':
   row.update(disposition='translated',reason='Pure branch decisions and preceding word write retained in prepare_child, with explicit frame supply and zero-child request. No allocator call or physical table reference is fabricated.',targets=[dict(path=target,anchor='pub machine prepare_child(')])
  elif path.endswith('/mapped_page_table.rs') and name in {'map_to_with_table_flags','unmap','update_flags','translate_page'}:
   name2={'map_to_with_table_flags':'map_leaf','unmap':'unmap_leaf','update_flags':'update_leaf_flags','translate_page':'translate_leaf'}[name]
   row.update(reason='Leaf decision component translated and tested in '+name2+'; complete parent traversal/orchestration and ordered partial-write plan composition remain pending.',targets=[dict(path=target,anchor='pub machine '+name2+'(')])
  elif path.endswith('/mod.rs') and name=='map_to':
   row.update(reason='Default parent flag selection extracted and tested; complete map call/allocator ownership remains pending.',targets=[dict(path=target,anchor='pub machine default_parent_flags(')])
  elif path.endswith('/mod.rs') and (name.startswith('self::') or name in {'mapped_page_table','offset_page_table','recursive_page_table','_ASSERT_OBJECT_SAFE','MapperAllSizes'}):
   row.update(disposition='omitted',reason='Rust re-export/module/trait-object or aggregate trait scaffolding does not define the pure decision API.')
 file.update(disposition='pending',reason='Pure child-creation and leaf decision components tested; complete mapper surface is not claimed.',targets=[dict(path=target,anchor='module mapping_plans;')])
text=json.dumps(doc,indent=2)+'\n'
if '--check' in sys.argv:assert p.read_text()==text,'stale reviewed mapping manifest'
else:p.write_text(text)
