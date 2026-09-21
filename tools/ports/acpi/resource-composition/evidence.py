#!/usr/bin/env python3
"""Reproduce complete source anchors without claiming the whole interpreter."""
import argparse,importlib.util,json
import fixtures
ROOT=fixtures.ROOT;UP=ROOT/'reference_code/rust-osdev/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',ROOT/'tools/ports/inventory.py')
api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def evidence():
 value=api.snapshot(UP,fixtures.PIN,['src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 value['scope']='Only the bounded ConcatRes result-construction component; whole interpreter methods remain pending.'
 for file in value['files'].values():
  file['reason']='Partial ConcatRes branch; generic Object/Store/context integration remains pending.'
  file['targets']=[{'path':'source/libraries/acpi/resource_composition/concatenate.omg','anchor':'pub machine compose'}]
  for row in file['symbols'].values():row['reason']='Outside this branch or whole interpreter method incomplete; no aggregate completion claim.'
 return value
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args()
 value=evidence();path=ROOT/'source/libraries/acpi/resource_composition/inventory.json'
 if a.check:assert json.loads(path.read_text())==value,'stale inventory'
 else:path.write_text(json.dumps(value,indent=2)+'\n')
 print(api.check(value,UP,ROOT))
if __name__=='__main__':main()
