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
 entry['targets']=[dict(path='source/libraries/acpi/aml/implicit_conversions.omg',anchor='pub machine '+name+'(')for name in ['convert']]
 entry['note']='Detached direct Integer/String/Buffer implicit conversion dispatch is available. Outer reference resolution, fields, destination storage and execution remain pending. This helper creates a new detached result and does not resize/truncate it to an existing named Buffer target.'
 value['primary_exports']={'ConversionTarget':['Integer','Buffer','String'],'ImplicitResult':['Failure(reason:ConversionFailure)','Integer(number:u64)','Buffer(length:u64,bytes:[u8;256])','String(length:u64,bytes:[u8;256])'],'convert':'direct ObjectStore ID + IntegerSize + ConversionTarget -> ImplicitResult'}
 source=(ROOT/'source/libraries/acpi/aml/implicit_conversions.omg').read_text()
 for anchor in ['pub data ConversionTarget','pub data ImplicitResult','case Failure(reason:ConversionFailure)','case Integer(number:u64)','case Buffer(length:u64,bytes:[u8;256])','case String(length:u64,bytes:[u8;256])','pub machine convert(']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/implicit-conversions-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,up,repository=ROOT))
if __name__=='__main__':main()
