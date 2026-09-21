#!/usr/bin/env python3
"""Retain aggregate comparison execution anchors beside the direct data adapter."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this direct comparison helper; aggregate execution remains pending.'
 for path,key in [('src/aml/mod.rs','1972:do_logical_op'),('src/aml/object.rs','377:aml_cmp')]:
  entry=value['files'][path]['symbols'][key];entry['targets']=[dict(path='source/libraries/acpi/aml/object_comparison.omg',anchor='pub machine compare(')];entry['note']='Direct Integer/String/Buffer comparison after primary right-hand conversion to the left type. Reference/Field evaluation, truth operators, Boolean result construction and context execution remain pending; pinned Buffer length-first ordering intentionally differs.'
 value['primary_exports']={'ObjectComparison':['Failure(reason:ConversionFailure)','Less','Equal','Greater'],'compare':'two direct ObjectStore IDs + IntegerSize -> ObjectComparison'}
 source=(ROOT/'source/libraries/acpi/aml/object_comparison.omg').read_text()
 for anchor in ['pub data ObjectComparison','case Failure(reason:ConversionFailure)','case Less;','case Equal;','case Greater;','pub machine compare(']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/object-comparison-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
