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
  for entry in item['symbols'].values():entry['reason']='Outside this Buffer/String Index construction helper; aggregate execution remains pending.'
 for path,key in [('src/aml/mod.rs','2283:do_index')]:
  entry=value['files'][path]['symbols'][key];entry['targets']=[dict(path='source/libraries/acpi/aml/byte_storage.omg',anchor='pub machine make_byte_index(')];entry['note']='Existing canonical byte_storage constructor: full byte/reference admission and index bounds before reserving two slots; BufferField plus RefOf publication leaves all byte blocks unchanged. Expanded whole-store checks and public identity probes add evidence, not a new implementation or aggregate closure.'
 value['primary_exports']={'make_byte_index':'Existing ObjectStore mutation -> ByteResult outcome, reference ID and8-bit length','reused_types':['model::ObjectStore','model::Value','model::ReferenceKind','byte_storage::ByteOutcome'],'reused_helpers':['byte_storage::read_bytes','object_references::unwrap_transparent']}
 source=(ROOT/'source/libraries/acpi/aml/byte_storage.omg').read_text()
 for anchor in ['pub machine make_byte_index(','read_bytes(','ReferenceKind::RefOf']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/byte-index-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
