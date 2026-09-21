#!/usr/bin/env python3
"""Selected pure method/status metadata map; whole Object implementation is pending."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();up=ROOT/'reference_code/rust-osdev/acpi'
 value=api.snapshot(up,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 mapping={'470:MethodFlags':'pub data MethodFacts','473:arg_count':'argument_count:raw&7','477:serialize':'serialized:(raw&8)!=0','481:sync_level':'sync_level:(raw>>4)&15','519:DeviceStatus':'pub data StatusFacts','522:present':'present:(raw&1)!=0','526:enabled':'enabled:(raw&2)!=0','530:show_in_ui':'show_in_ui:(raw&4)!=0','534:functioning':'functioning:(raw&8)!=0','540:battery_present':'battery_present:(raw&16)!=0'}
 for key,entry in value['files']['src/aml/object.rs']['symbols'].items():
  entry['reason']='Outside this metadata slice; no whole Object implementation claim.'
  if key in mapping:entry.update(disposition='translated',reason='Pure initialized metadata decode; no synchronization, status validation, enumeration or hardware policy.',targets=[dict(path='source/libraries/acpi/aml/object_metadata.omg',anchor=mapping[key])])
 path=ROOT/'source/libraries/acpi/aml/object-metadata-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,up,repository=ROOT))
if __name__=='__main__':main()
