#!/usr/bin/env python3
"""Retain the aggregate Store anchor beside direct BufferField source composition."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');a=p.parse_args()
 v=api.snapshot(a.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 for f in v['files'].values():
  for e in f['symbols'].values():e['reason']='Outside direct BufferField source Store composition; aggregate execution remains pending.'
 e=v['files']['src/aml/mod.rs']['symbols']['2406:do_store'];e['targets']=[dict(path='source/libraries/acpi/aml/buffer_field_store.omg',anchor='pub machine store_value(')];e['note']='Direct Integer/Buffer/String source admission and atomic fixed BufferField write, preserving shared source/backing identity through a detached snapshot. Target/reference evaluation, other destination types and opcode execution remain pending.'
 v['primary_exports']={'store_value':'direct field/source IDs + IntegerSize and mutable canonical ObjectStore -> ByteResult; failure atomic, success identifies backing'}
 assert 'pub machine store_value('in(ROOT/'source/libraries/acpi/aml/buffer_field_store.omg').read_text()
 path=ROOT/'source/libraries/acpi/aml/buffer-field-store-inventory.json';text=json.dumps(v,indent=2,sort_keys=True)+'\n'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(v,a.checkout,repository=ROOT))
if __name__=='__main__':main()
