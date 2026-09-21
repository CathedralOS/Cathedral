import argparse,importlib.util,json
from pathlib import Path
import fixtures
ROOT=fixtures.ROOT
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();up=ROOT/'reference_code/rust-osdev/acpi';value=api.snapshot(up,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/namespace.rs'],'https://github.com/rust-osdev/acpi')
 for key,row in value['files']['src/aml/namespace.rs']['symbols'].items():
  row['reason']='Outside this namespace-removal slice.'
  if key=='156:remove_level':row.update(disposition='translated',reason='Bounded transactional flat-entry subtree removal, preserving parent-level same-path object and stable object IDs; stricter structural admission. No AML Unload implementation or reclamation.',targets=[dict(path='source/libraries/acpi/aml/namespace_removal.omg',anchor='pub machine remove_level')])
 path=ROOT/'source/libraries/acpi/aml/namespace-removal-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(api.check(value,up,repository=ROOT))
if __name__=='__main__':main()
