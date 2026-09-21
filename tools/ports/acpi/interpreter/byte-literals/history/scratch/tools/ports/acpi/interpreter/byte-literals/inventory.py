#!/usr/bin/env python3
import argparse,importlib.util,json
from pathlib import Path
import fixtures
CANONICAL=Path('/Users/zcanann/Documents/projects/Cathedral');UP=CANONICAL/'reference_code/rust-osdev/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',CANONICAL/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def inventory():
 value=api.snapshot(UP,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/mod.rs'],'https://github.com/rust-osdev/acpi')
 value['scope']='Pure String and constant-size Buffer literal preflight component only; no executor integration or namespace publication.'
 for file in value['files'].values():
  file['reason']='Whole interpreter file remains pending; selected literal component is implemented separately.'
  for key,row in file['symbols'].items():
   row['reason']='Outside this isolated literal component, or aggregate operation still incomplete.'
   if key.endswith(':do_execute_method'):
    row['reason']='Partial StringPrefix parse and Buffer constant-size preflight only. All opcode execution/retirement, dynamic BufferSize TermArg, Package, namespace and handler effects remain outside this helper.'
    row['targets']=[{'path':'source/libraries/acpi/interpreter/execution/byte_literals.omg','anchor':'pub machine preflight_byte_literal'}]
    row['components']=[{'upstream_lines':'1234-1244','meaning':'ASCII String source parsing and bounded byte admission; stricter than pinned UTF8 unwrap.'},{'upstream_lines':'1261-1269,718-743','meaning':'Constant BufferSize only; source envelope/width normalization, spec max(declared,initializer) and zero padding. Dynamic evaluation and retirement remain pending.'}]
 return value
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();v=inventory();path=fixtures.ROOT/'source/libraries/acpi/interpreter/execution/byte-literals-inventory.json'
 if a.check:assert json.loads(path.read_text())==v
 else:path.write_text(json.dumps(v,indent=2)+'\n')
 print(api.check(v,UP,fixtures.ROOT))
if __name__=='__main__':main()
