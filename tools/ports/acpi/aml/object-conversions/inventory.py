#!/usr/bin/env python3
import argparse,importlib.util,json
from pathlib import Path
import fixtures
CANONICAL=fixtures.ROOT;UP=CANONICAL/'reference_code/rust-osdev/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',CANONICAL/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def inventory():
 value=api.snapshot(UP,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/object.rs','src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 value['scope']='Direct canonical Value integer/buffer conversion preflight only; strict explicit numeric policy and named pinned buffer compatibility, no context/target dispatch.'
 for file in value['files'].values():
  file['reason']='Whole interpreter file remains pending; selected literal component is implemented separately.'
  for key,row in file['symbols'].items():
   row['reason']='Outside this isolated literal component, or aggregate operation still incomplete.'
   name=key.split(':',1)[1]
   if name in ['to_integer','to_buffer','do_to_integer','do_to_buffer']:
    target='to_integer'if name.endswith('integer')else'to_buffer'
    row['reason']='Partial bounded direct canonical Value conversion kernel; full context/reference/target semantics and wider BufferField conversion remain outside this slice. String explicit strict policy and pinned unterminated-buffer compatibility are separately named.'
    row['targets']=[{'path':'source/libraries/acpi/aml/object_conversions.omg','anchor':'pub machine '+target}]
 return value
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();v=inventory();path=fixtures.ROOT/'source/libraries/acpi/aml/object-conversions-inventory.json'
 if a.check:assert json.loads(path.read_text())==v
 else:path.write_text(json.dumps(v,indent=2)+'\n')
 print(api.check(v,UP,fixtures.ROOT))
if __name__=='__main__':main()
