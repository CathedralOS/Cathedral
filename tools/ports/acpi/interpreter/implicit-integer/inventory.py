#!/usr/bin/env python3
"""Keep upstream aggregate conversions pending beside the primary-rule helper."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args();up=ROOT/'reference_code/rust-osdev/acpi'
 value=api.snapshot(up,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this primary-rule helper; aggregate Rust operations remain pending.'
 entry=value['files']['src/aml/mod.rs']['symbols']['2406:do_store']
 entry['targets']=[dict(path='source/libraries/acpi/interpreter/implicit_integer.omg',anchor='pub machine from_string(')]
 entry['note']='Only the primary Table 19.7 String-to-Integer rule is available. Generic source resolution, conversion selection, target storage and execution remain pending; the pinned decimal parser is not the specification oracle.'
 value['primary_exports']={'Conversion':['Failure(reason:Error)','Integer(value:u64)'],'Error':['Capacity','Empty','Encoding'],'from_string':'IntegerSize, &[u8;256], u64 -> Conversion'}
 source=(ROOT/'source/libraries/acpi/interpreter/implicit_integer.omg').read_text()
 for anchor in ['pub data Conversion','case Failure(reason:Error)','case Integer(value:u64)','pub data Error','case Capacity','case Empty','case Encoding','pub machine from_string(']:assert anchor in source
 path=ROOT/'source/libraries/acpi/interpreter/implicit-integer-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,up,repository=ROOT))
if __name__=='__main__':main()
