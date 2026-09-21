#!/usr/bin/env python3
"""Reproduce source mapping for bounded reference resolution and query components."""
import argparse,importlib.util,json
import fixtures
ROOT=fixtures.ROOT;UP=ROOT/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
spec=importlib.util.spec_from_file_location('port_inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def evidence():
 value=api.snapshot(UP,PIN,['src/aml/mod.rs'],'https://github.com/rust-osdev/acpi');value['scope']='Bounded canonical mixed reference resolver; numeric ObjectType and validated SizeOf components, without generic opcode/context integration.'
 for file in value['files'].values():
  file['reason']='Selected bounded canonical object-query components; whole interpreter remains pending.'
  for key,row in file['symbols'].items():
   name=key.split(':',1)[1];row['reason']='Outside this slice or whole interpreter operation still incomplete.'
   if name in ['resolve_name_path','object_type','do_size_of']:
    machine={'resolve_name_path':'resolve_object','object_type':'object_type','do_size_of':'size_of'}[name]
    row['targets']=[{'path':'source/libraries/acpi/aml/object_queries.omg','anchor':'pub machine '+machine}]
    row['reason']='Bounded component; scopes and absent Value payloads / opcode retirement remain separate.'
    if name=='resolve_name_path':row.update(disposition='translated',reason='Mixed lexical/ID resolution with one64-inspection budget and cycle bitmap; explicit bounded adaptation.')
 return value
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();value=evidence();path=ROOT/'source/libraries/acpi/aml/object-queries-inventory.json'
 if a.check:assert json.loads(path.read_text())==value,'source map drift'
 else:path.write_text(json.dumps(value,indent=2)+'\n')
 print(api.check(value,UP,ROOT))
if __name__=='__main__':main()
