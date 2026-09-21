#!/usr/bin/env python3
import argparse,importlib.util,json
from pathlib import Path
import fixtures
CANONICAL=Path("/Users/zcanann/Documents/projects/Cathedral");UP=CANONICAL/'reference_code/rust-osdev/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',CANONICAL/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def inventory():
 value=api.snapshot(UP,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 value['scope']='Direct Integer/String/Buffer explicit numeric String result only; no operand/target/context dispatch.'
 for file in value['files'].values():
  file['reason']='Whole interpreter operations remain pending; this is a partial direct-value composition.'
  for key,row in file['symbols'].items():
   row['reason']='Outside this component, or aggregate operation remains incomplete.'
   if key.split(':',1)[1]=='do_to_dec_hex_string':
    row['reason']='Partial direct Integer/String/Buffer composition with selected-width normalization, exact pinned explicit numeric presentation, strict canonical String identity and bounded capacity. Operand evaluation/reference handling, targets, stores, context contribution and retirement remain pending.'
    row['targets']=[{'path':'source/libraries/acpi/aml/object_numeric_strings.omg','anchor':'pub machine convert'}]
 return value
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();v=inventory();path=fixtures.ROOT/'source/libraries/acpi/aml/object-numeric-strings-inventory.json'
 if a.check:assert json.loads(path.read_text())==v
 else:path.write_text(json.dumps(v,indent=2)+'\n')
 print(api.check(v,UP,fixtures.ROOT))
if __name__=='__main__':main()
