#!/usr/bin/env python3
"""Check the pinned field-syntax supplement and original fixture mutation anchors."""
import hashlib
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[5]
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'tools/ports'))
import inventory

def main():
 checkout=ROOT/'reference_code/rust-osdev/acpi'
 manifest=inventory.read_json(ROOT/'source/libraries/acpi/aml/fields/inventory.json')
 print(json.dumps(inventory.check(manifest,checkout,ROOT),indent=2))
 provenance=inventory.read_json(HERE/'provenance.json')
 assert provenance['revision']==manifest['upstream']['revision']
 for path,digest in provenance['license_and_manifest_sha256'].items():
  assert hashlib.sha256((checkout/path).read_bytes()).hexdigest()==digest,path
 for path,entry in provenance['test_assets'].items():
  assert hashlib.sha256((checkout/path).read_bytes()).hexdigest()==entry['sha256'],path
 cases=inventory.read_json(HERE/'cases.json')
 coverage=inventory.read_json(HERE/'coverage.json')
 for form,item in coverage['forms'].items():
  target=item['target']
  assert target['anchor'] in (ROOT/target['path']).read_text(),form
  assert item['reason']
 for name,item in cases.items():
  assert (HERE/'cases'/f'{name}.omg').read_text().count(item['mutation'][0])==1,name
 print(f'{len(cases)} unique body controls; four upstream test assets retained as metadata only.')
 print('Syntax coverage only; namespace installation and runtime fields remain pending.')
if __name__=='__main__':main()
