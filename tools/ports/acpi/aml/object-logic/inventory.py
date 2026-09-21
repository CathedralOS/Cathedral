#!/usr/bin/env python3
"""Retain aggregate logical execution anchor beside its direct data component."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this direct logical data helper; aggregate execution remains pending.'
 entry=value['files']['src/aml/mod.rs']['symbols']['1972:do_logical_op'];entry['targets']=[dict(path='source/libraries/acpi/aml/object_logic.omg',anchor='pub machine binary('),dict(path='source/libraries/acpi/aml/object_logic.omg',anchor='pub machine negate(')];entry['note']='Direct Integer/String/Buffer truth and six relational operations produce width-normalized AML Boolean Integers after primary conversions. Reference/Field evaluation, operand acquisition, context contribution and opcode retirement remain pending.'
 value['primary_exports']={'LogicalResult':['Failure(reason:ConversionFailure)','Integer(number:u64)'],'binary':'two direct IDs + IntegerSize + Logical -> LogicalResult','negate':'direct ID + IntegerSize -> LogicalResult'}
 source=(ROOT/'source/libraries/acpi/aml/object_logic.omg').read_text()
 for anchor in ['pub data LogicalResult','case Failure(reason:ConversionFailure)','case Integer(number:u64)','pub machine binary(','pub machine negate(']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/object-logic-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
