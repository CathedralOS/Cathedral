#!/usr/bin/env python3
"""Bind complete pinned source anchors while retaining generic operations pending."""
import argparse,hashlib,importlib.util,json
from pathlib import Path
import fixtures
ROOT=fixtures.ROOT;HERE=fixtures.HERE;UP=ROOT/'reference_code/rust-osdev/acpi';DEST=ROOT/'source/libraries/acpi/interpreter'
spec=importlib.util.spec_from_file_location('port_inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def evidence():
 inv=api.snapshot(UP,fixtures.PIN,['src/aml/object.rs','src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 inv['scope']='String numeric parsing and integer/byte-buffer formatting components only; full Object/context/target operations remain pending.'
 for path,file in inv['files'].items():
  file['reason']='Complete source inventoried; this component implements only selected pure conversion expressions.'
  file['targets']=[{'path':'source/libraries/acpi/interpreter/string_numbers.PORT.md','anchor':'## Source map'}]
  for key,row in file['symbols'].items():
   name=key.split(':',1)[1];row['reason']='Outside this narrow component, or generic object/context/target behavior still pending.'
   names={'to_integer':'pub machine string_to_integer','do_to_dec_hex_string':'pub machine integer_to_string','do_to_integer':'pub machine string_to_integer'}
   if name in names:row.update(targets=[{'path':'source/libraries/acpi/interpreter/string_numbers.omg','anchor':names[name]}],reason='Partial pure expression translation only. Generic Object resolution, reference semantics, target stores and/or method context retirement remain pending; no whole-operation translation claim.')
 inv['license_sha256']={p:hashlib.sha256((UP/p).read_bytes()).hexdigest()for p in ['LICENCE-MIT','LICENCE-APACHE','Cargo.toml']}
 inv['primary_references']=[{'url':'https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#tointeger-convert-data-to-integer','sections':['19.3.5','19.6.139','19.6.140','19.6.141'],'reviewed':'2026-09-20','meaning':'Explicit conversion forms; malformed lexical rejection/compatibility profiles and finite capacities are separately named Cathedral policy.'}]
 return inv
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();value=evidence();text=json.dumps(value,indent=2,sort_keys=True)+'\n';path=DEST/'string-numbers-inventory.json'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,UP,ROOT))
if __name__=='__main__':main()
