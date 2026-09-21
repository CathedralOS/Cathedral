#!/usr/bin/env python3
"""Retain the aggregate Index anchor beside bounded Package reference construction."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this Package Index construction helper; aggregate execution remains pending.'
 for path,key in [('src/aml/mod.rs','2283:do_index')]:
  entry=value['files'][path]['symbols'][key];entry['targets']=[dict(path='source/libraries/acpi/aml/package_index.omg',anchor='pub machine make_package_index(')];entry['note']='Direct Package ID and evaluated unsigned index; complete existing linked-chain admission followed by one fresh RefOf wrapper preserving member identity. No operand evaluation, element resolution, target Store or retirement. Existing byte arena, including fresh reference slot backing, is unchanged.'
 value['primary_exports']={'make_package_index':'ObjectStore mutation after full admission -> canonical Read outcome and fresh stable ID','reused_types':['model::ObjectStore','model::Read','model::ReferenceKind'],'reused_helpers':['object_references::package_element','object_references::allocate_reference']}
 source=(ROOT/'source/libraries/acpi/aml/package_index.omg').read_text()
 for anchor in ['pub machine make_package_index(','package_element(','allocate_reference(','ReferenceKind::RefOf']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/package-index-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
