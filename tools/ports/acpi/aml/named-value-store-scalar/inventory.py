#!/usr/bin/env python3
"""Retain aggregate replacement/Store anchors beside direct named-value preparation."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');args=p.parse_args()
 value=api.snapshot(args.checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 for item in value['files'].values():
  for entry in item['symbols'].values():entry['reason']='Outside this direct named-value Store helper; aggregate execution remains pending.'
 for path,key in [('src/aml/mod.rs','2406:do_store'),('src/aml/object.rs','317:replace_with_implicit_casting')]:
  entry=value['files'][path]['symbols'][key];entry['targets']=[dict(path='source/libraries/acpi/aml/named_value_store.omg',anchor=anchor)for anchor in ['pub machine store_value(','pub machine store_integer(','pub machine admit_destination(']];entry['note']='Destination-first direct allocated Integer/String/positive-extent Buffer Store with canonical implicit conversion and atomic payload/block replacement. Named-target policy chosen by caller; no reference/field/execution dispatch. Zero Buffer extent, empty String-to-Buffer and Buffer-to-Buffer remain explicit local exclusions.'
 value['scalar_extension']={'store_integer':'already-evaluated u64 normalized to active IntegerSize; no source object slot','admit_destination':'failure-first ordinary Target(type,extent) metadata; both mutation entry points re-admit current store','numeric_reuse':['integers::normalize','implicit_strings::from_integer','conversions::integer_to_buffer','conversions::mid']}
 value['primary_exports']={'store_value':'direct allocated destination/source IDs -> Failure(ConversionFailure) or Stored ID; fixed named-target caller policy','reused_types':['model::ObjectStore','model::Value','model::ByteBlock','implicit_conversions::ImplicitResult','object_conversions::ConversionFailure','integers::IntegerSize'],'reused_helpers':['byte_storage::read_bytes','implicit_conversions::convert','buffer_target_values::prepare_buffer_extent']}
 source=(ROOT/'source/libraries/acpi/aml/named_value_store.omg').read_text()
 for anchor in ['pub machine store_value(','pub data StoreResult','prepare_buffer_extent(','store.bytes.blocks[destination]=ByteBlock {}']:assert anchor in source
 path=ROOT/'source/libraries/acpi/aml/named-value-store-scalar-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,args.checkout,repository=ROOT))
if __name__=='__main__':main()
