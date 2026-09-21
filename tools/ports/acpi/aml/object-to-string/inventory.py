#!/usr/bin/env python3
"""Retain the aggregate ToString execution anchor beside its direct Buffer adapter."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this direct ToString helper; aggregate execution remains pending.'
 entry=value['files']['src/aml/mod.rs']['symbols']['2068:do_to_string'];entry['targets']=[dict(path='source/libraries/acpi/aml/object_to_string.omg',anchor='pub machine to_string(')];entry['note']='Direct Buffer ObjectStore admission and bounded ASCII String composition only. Complete backing admission precedes prefix selection; NUL excluded. No reference evaluation, allocation, target Store or context retirement; aggregate remains pending. Pin includes NUL and accepts valid UTF8.'
 value['primary_exports']={'StringResult':['Failure(reason:ConversionFailure)','String(length:u64,bytes:[u8;256])'],'to_string':'direct Buffer ObjectStore ID + evaluated unsigned maximum -> detached StringResult'}
 source=(ROOT/'source/libraries/acpi/aml/object_to_string.omg').read_text()
 for anchor in ['pub data StringResult','case Failure(reason:ConversionFailure)','case String(length:u64,bytes:[u8;256])','pub machine to_string(']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/object-to-string-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
