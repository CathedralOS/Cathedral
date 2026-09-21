#!/usr/bin/env python3
"""Exact pinned reference component source map; generic object operations pending."""
import argparse,hashlib,importlib.util,json
from pathlib import Path
import fixtures,reference
ROOT=fixtures.ROOT;UP=reference.UP;DEST=ROOT/'source/libraries/acpi/aml'
spec=importlib.util.spec_from_file_location('port_inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def evidence():
 inv=api.snapshot(UP,reference.PIN,['src/aml/object.rs','src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 inv['scope']='Canonical stable-ID reference, linked package selection and shallow payload-copy components. Full generic Object/Store/CopyObject/opcode/context/ownership integration remains pending.'
 target='source/libraries/acpi/aml/object_references.omg'
 for path,file in inv['files'].items():
  file.update(reason='Complete pinned source inventoried; selected reference/value-copy components only.',targets=[{'path':'source/libraries/acpi/aml/object-references.PORT.md','anchor':'## Source map'}])
  for key,row in file['symbols'].items():
   name=key.split(':',1)[1];line=int(key.split(':',1)[0]);row['reason']='Outside this component; broad object/context/target semantics remain pending in their owning slices.'
   if path=='src/aml/object.rs'and name in ['unwrap_reference','unwrap_transparent_reference','ReferenceKind']:
    anchor={'unwrap_reference':'pub machine unwrap_all','unwrap_transparent_reference':'pub machine unwrap_transparent','ReferenceKind':'pub data ReferenceKind'}[name];row.update(disposition='translated',reason='Canonical stable-ID equivalent for valid acyclic references; named bounded cycle/dangling/budget rejection policies added.',targets=[{'path':'source/libraries/acpi/aml/model.omg'if name=='ReferenceKind'else target,'anchor':anchor}])
   elif path=='src/aml/object.rs'and name in ['new','wrap','WrappedObject','Object']:
    row.update(reason='Reference identity/allocation and inert payload-copy component only; Arc/UnsafeCell ownership, mutable byte backing, token access and full Object behavior remain pending.',targets=[{'path':target,'anchor':'pub machine allocate_reference'},{'path':target,'anchor':'pub machine copy_value'}])
   elif path=='src/aml/mod.rs'and name=='do_index':row.update(reason='Package selection component only, with full bounded linked-chain validation; buffer/string fields, target store, expression/context retirement and opcode integration remain pending.',targets=[{'path':target,'anchor':'pub machine package_element'}])
   elif path=='src/aml/mod.rs'and name in ['do_store','do_copy_object']:row.update(reason='A shallow payload copy is not these operations: target-kind dispatch, implicit conversions, region/service handling and retirement remain pending.',targets=[{'path':target,'anchor':'pub machine copy_value'}])
 inv['license_sha256']={name:hashlib.sha256((UP/name).read_bytes()).hexdigest()for name in ['LICENCE-MIT','LICENCE-APACHE','Cargo.toml']}
 return inv
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();value=evidence();text=json.dumps(value,indent=2,sort_keys=True)+'\n';out=DEST/'object-references-inventory.json'
 if a.check:assert out.read_text()==text
 else:out.write_text(text)
 print(api.check(value,UP,ROOT))
if __name__=='__main__':main()
