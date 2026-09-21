#!/usr/bin/env python3
"""Retain aggregate math anchors beside direct canonical operand computation."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this direct arithmetic component; aggregate execution remains pending.'
 for key,name in [('1896:do_binary_maths','binary'),('1931:do_unary_maths','unary'),('2233:do_from_bcd','unary'),('2250:do_to_bcd','unary')]:
  entry=value['files']['src/aml/mod.rs']['symbols'][key];entry['targets']=[dict(path='source/libraries/acpi/aml/object_maths.omg',anchor='pub machine '+name+'(')];entry['note']='Direct Integer/String/Buffer operands undergo primary implicit Integer conversion and existing width-normalized mathematics. Semantic failures and detached quotient/remainder are retained. Reference/Field evaluation, target stores, context contribution and retirement remain pending.'
 value['primary_exports']={'Unary':['BitwiseNot','FindSetLeft','FindSetRight','FromBcd','ToBcd'],'MathFailure':['Conversion(reason:ConversionFailure)','DivideByZero','InvalidBcd','Overflow'],'MathResult':['Failure(reason:MathFailure)','Integer(number:u64)','Division(quotient:u64,remainder:u64)'],'binary':'two direct IDs + IntegerSize + Binary -> MathResult','unary':'direct ID + IntegerSize + Unary -> MathResult'}
 source=(ROOT/'source/libraries/acpi/aml/object_maths.omg').read_text()
 for anchor in ['pub data Unary','pub data MathFailure','pub data MathResult','case Failure(reason:MathFailure)','case Division(quotient:u64,remainder:u64)','pub machine binary(','pub machine unary(']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/object-maths-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
