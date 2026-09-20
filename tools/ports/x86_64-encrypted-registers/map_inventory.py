#!/usr/bin/env python3
"""Encryption-profile composition overlay for pinned register expressions."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
checkout=ROOT/'reference_code/rust-osdev/x86_64';pin='cc35c876d3badb57df54a66e22f7768a52be95f2'
paths=['src/registers/control.rs','src/registers/model_specific.rs']
data=shared.snapshot(checkout,pin,paths,'https://github.com/rust-osdev/x86_64');target='source/libraries/x86_64/encrypted_register_operands.omg'
selectors={'src/registers/control.rs':{348:'cr3_observed',376:'cr3_flags_operand',390:'cr3_pcid_operand',405:'cr3_pcid_operand',418:'cr3_raw_operand',423:'cr3_raw_operand'},'src/registers/model_specific.rs':{689:'apic_observed',705:'apic_preserving_operand',723:'apic_raw_operand'}}
for name,file in data['files'].items():
 assert set(selectors[name]).issubset({int(key.split(':',1)[0]) for key in file['symbols']}),name
 file.update(disposition='pending',reason='Only pure profile-sensitive observation/operand expressions; live register APIs and unrelated components remain outside this overlay.',targets=[{'path':target,'anchor':'module encrypted_register_operands;'}])
 for key,row in file['symbols'].items():
  line=int(key.split(':',1)[0]);machine=selectors[name].get(line)
  if machine:row.update(disposition='translated',reason='Encryption-aware numeric expression component only. Caller supplies raw observation/frame; no CPU read/write, support discovery or authority.',targets=[{'path':target,'anchor':'pub machine '+machine}])
  else:row.update(disposition='pending',reason='Outside the profile-sensitive component overlay; prior register inventories retain their own scopes.')
out=ROOT/'source/libraries/x86_64/encrypted-registers-inventory.json';text=json.dumps(data,indent=2)+'\n'
if '--check' in sys.argv:assert out.read_text()==text,'stale encrypted register inventory'
else:out.write_text(text)
print(shared.check(data,checkout))
