#!/usr/bin/env python3
import argparse,importlib.util,json
from pathlib import Path
import fixtures
CANONICAL=fixtures.ROOT;UP=CANONICAL/'reference_code/rust-osdev/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',CANONICAL/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def inventory():
 value=api.snapshot(UP,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 value['scope']='Direct bounded basic/description Concatenate composition; no reference evaluation, target mutation or opcode retirement.'
 for file in value['files'].values():
  file['reason']='Aggregate interpreter operations and nested formatter remain pending.'
  for key,row in file['symbols'].items():
   row['reason']='Outside this detached composition, or aggregate operation remains incomplete.'
   if key.split(':',1)[1]in ['do_concat','resolve_as_string']:
    row['reason']='Partial primary basic pairs plus String/Buffer-left descriptions and description-left String composition; Integer-left descriptions are locally excluded. References/names, unrepresented cases, field/method evaluation, target/context and retirement remain pending.'
    row['targets']=[{'path':'source/libraries/acpi/aml/object_concat_described.omg','anchor':'pub machine concatenate'}]
 return value
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();v=inventory();path=fixtures.ROOT/'source/libraries/acpi/aml/object-concat-described-inventory.json'
 if a.check:assert json.loads(path.read_text())==v
 else:path.write_text(json.dumps(v,indent=2)+'\n')
 print(api.check(v,UP,fixtures.ROOT))
if __name__=='__main__':main()
