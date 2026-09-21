#!/usr/bin/env python3
"""Retain aggregate replacement/Store anchors beside positive-extent preparation."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this positive-extent Buffer preparation helper; aggregate execution remains pending.'
 for path,key in [('src/aml/mod.rs','2406:do_store'),('src/aml/object.rs','317:replace_with_implicit_casting')]:
  entry=value['files'][path]['symbols'][key];entry['targets']=[dict(path='source/libraries/acpi/aml/buffer_target_values.omg',anchor='pub machine prepare_buffer_extent(')];entry['note']='Detached positive explicit extent1..256, direct Integer/nonemptyString preparation under primary Table19.7. No target presence/provenance, reference evaluation, mutation or Store; zero extent/emptyString/same-typeBuffer policies excluded explicitly. Pin replacement always resizes to raw source bytes.'
 value['primary_exports']={'prepare_buffer_extent':'direct sourceID + IntegerSize + explicit positive extent -> canonical ImplicitResult Failure/Buffer','reused_types':['implicit_conversions::ImplicitResult','object_conversions::ConversionFailure','integers::IntegerSize']}
 source=(ROOT/'source/libraries/acpi/aml/buffer_target_values.omg').read_text()
 for anchor in ['pub machine prepare_buffer_extent(','ImplicitResult::Buffer','ConversionFailure::Bounds','ConversionFailure::Empty']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/buffer-target-values-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
