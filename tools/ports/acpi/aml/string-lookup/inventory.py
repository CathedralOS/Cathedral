#!/usr/bin/env python3
"""Map direct String-name lookup without claiming complete DerefOf."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this direct String-name lookup helper; aggregate execution remains pending.'
 for path,key in [('src/aml/mod.rs','2383:do_deref_of')]:
  entry=value['files'][path]['symbols'][key];entry['targets']=[dict(path='source/libraries/acpi/aml/string_lookup.omg',anchor='pub machine lookup(')];entry['note']='Direct canonical String storage admission, ASL textual name parsing and existing scoped namespace search return a stable object identity and resolved path. Reference/context evaluation, target reading and full DerefOf execution remain pending.'
 value['primary_exports']={'StringLookup':['Failure(reason:LookupFailure)','Found(object:u64,path:Path)'],'lookup':'direct String ObjectStore ID + current Path scope -> stable identity without target evaluation'}
 source=(ROOT/'source/libraries/acpi/aml/string_lookup.omg').read_text()
 for anchor in ['pub data StringLookup','case Failure(reason:LookupFailure)','case Found(object:u64,path:Path)','pub machine lookup(']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/string-lookup-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
