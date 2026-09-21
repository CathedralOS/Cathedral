#!/usr/bin/env python3
"""Bind complete pinned source anchors while retaining generic operations pending."""
import argparse,hashlib,importlib.util,json
from pathlib import Path
import fixtures
ROOT=fixtures.ROOT;HERE=fixtures.HERE;UP=ROOT/'reference_code/rust-osdev/acpi';DEST=ROOT/'source/libraries/acpi/interpreter'
spec=importlib.util.spec_from_file_location('port_inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def evidence():
 inv=api.snapshot(UP,fixtures.PIN,['src/aml/object.rs','src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 inv['scope']='Bounded same-type concatenation components only; full Object/context/target operations remain pending.'
 for path,file in inv['files'].items():
  file['reason']='Complete source inventoried; this component implements only selected same-type concatenation expressions.'
  file['targets']=[{'path':'source/libraries/acpi/interpreter/byte_concat.PORT.md','anchor':'## Source map'}]
  for key,row in file['symbols'].items():
   name=key.split(':',1)[1];row['reason']='Outside this narrow component, or generic object/context/target behavior still pending.'
   names={'do_concat':'pub machine concat_integers'}
   if name in names:row.update(targets=[{'path':'source/libraries/acpi/interpreter/byte_concat.omg','anchor':names[name]}],reason='Partial pure expression translation only. Generic Object resolution, reference semantics, target stores and/or method context retirement remain pending; no whole-operation translation claim.')
   if name=='do_concat':
    row['targets']=[{'path':'source/libraries/acpi/interpreter/byte_concat.omg','anchor':'pub machine '+operation}for operation in ['concat_integers','concat_buffers','concat_strings']]
    row['implemented_components']=['already-converted integer pair little-endian bytes','bounded same-type buffer append','bounded ASCII/NUL-free string extent append']
    row['remaining_components']=['generic implicit conversions','type-name formatting','transparent reference/object resolution','target Store/coercion','method result contribution and opcode retirement']
 inv['license_sha256']={p:hashlib.sha256((UP/p).read_bytes()).hexdigest()for p in ['LICENCE-MIT','LICENCE-APACHE','Cargo.toml']}
 inv['primary_references']=[{'url':'https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#concatenate-concatenate-data','sections':['19.6.12','Table 19.30'],'reviewed':'2026-09-20','meaning':'Already-converted integer/buffer/string concatenation only; implicit conversion, type-name formatting, target storage and opcode integration remain pending.'}]
 return inv
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();value=evidence();text=json.dumps(value,indent=2,sort_keys=True)+'\n';path=DEST/'byte-concat-inventory.json'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,UP,ROOT))
if __name__=='__main__':main()
