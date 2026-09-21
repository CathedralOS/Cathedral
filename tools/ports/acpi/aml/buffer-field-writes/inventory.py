#!/usr/bin/env python3
import argparse,importlib.util,json
from pathlib import Path
import fixtures
CANONICAL=fixtures.ROOT;UP=CANONICAL/'reference_code/rust-osdev/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',CANONICAL/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def inventory():
 value=api.snapshot(UP,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 value['scope']='Atomic bounded direct BufferField write from already-converted bytes; whole-store error atomicity and preserved tails, no runtime/Store/service dispatch.'
 for file in value['files'].values():
  file['reason']='Only write_buffer_field is translated by this component; generic Object and remaining operations are outside scope.'
  for key,row in file['symbols'].items():
   row['reason']='Outside this atomic byte-write component.'
   if key.split(':',1)[1]=='write_buffer_field':
    row['disposition']='translated';row['targets']=[{'path':'source/libraries/acpi/aml/buffer_field_writes.omg','anchor':'pub machine write'}]
    row['reason']='Bounded direct canonical field write using payload/backing snapshots and existing bit/encoding helpers; zero/out-of-range rejection, atomic String validation and finite 256-byte profile. Actual public method called with the real Interpreter-created locked token; no fabricated authority or runtime/Store dispatch.'
 return value
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();v=inventory();out=fixtures.ROOT/'source/libraries/acpi/aml/buffer-field-writes-inventory.json'
 if a.check:assert json.loads(out.read_text())==v
 else:out.write_text(json.dumps(v,indent=2)+'\n')
 print(api.check(v,UP,fixtures.ROOT))
if __name__=='__main__':main()
