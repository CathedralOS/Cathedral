#!/usr/bin/env python3
"""Keep full Concatenate dispatch pending beside direct basic-data composition."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this direct basic-data concatenation helper; aggregate execution remains pending.'
 for path,key in [('src/aml/mod.rs','2171:do_concat'),('src/aml/mod.rs','2176:resolve_as_string')]:
  entry=value['files'][path]['symbols'][key];entry['targets']=[dict(path='source/libraries/acpi/aml/object_concat.omg',anchor='pub machine concatenate(')];entry['note']='Basic Integer/String/Buffer concatenation after primary right-hand conversion. Other-type name formatting, reference/context policy, target coercion, object installation and opcode execution remain pending.'
 value['primary_exports']={'Concatenated':['Failure(reason:ConversionFailure)','Buffer(length:u64,bytes:[u8;256])','String(length:u64,bytes:[u8;256])'],'concatenate':'two direct basic-data ObjectStore IDs + IntegerSize -> Concatenated'}
 source=(ROOT/'source/libraries/acpi/aml/object_concat.omg').read_text()
 for anchor in ['pub data Concatenated','case Failure(reason:ConversionFailure)','case Buffer(length:u64,bytes:[u8;256])','case String(length:u64,bytes:[u8;256])','pub machine concatenate(']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/object-concat-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
