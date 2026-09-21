#!/usr/bin/env python3
import argparse,importlib.util,json
from pathlib import Path
import fixtures
CANONICAL=fixtures.ROOT;UP=CANONICAL/'reference_code/rust-osdev/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',CANONICAL/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def inventory():
 value=api.snapshot(UP,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 value['scope']='Detached bounded direct BufferField read: primary integer-bit-width shape, strict metadata validation; no runtime or Field Unit services.'
 for file in value['files'].values():
  file['reason']='Only read_buffer_field is translated by this component; generic Object and remaining operations are outside scope.'
  for key,row in file['symbols'].items():
   row['reason']='Outside this detached field-read component.'
   if key.split(':',1)[1]=='read_buffer_field':
    row['disposition']='translated';row['targets']=[{'path':'source/libraries/acpi/aml/buffer_field_values.omg','anchor':'pub machine read'}]
    row['reason']='Bounded direct canonical field read, shared backing byte snapshots and existing bit helpers; primary 32/64-bit shape correction, zero/out-of-range rejection, strict string storage encoding and finite 256-byte limit documented. No pointer, runtime dispatch or region access.'
 return value
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();v=inventory();out=fixtures.ROOT/'source/libraries/acpi/aml/buffer-field-values-inventory.json'
 if a.check:assert json.loads(out.read_text())==v
 else:out.write_text(json.dumps(v,indent=2)+'\n')
 print(api.check(v,UP,fixtures.ROOT))
if __name__=='__main__':main()
