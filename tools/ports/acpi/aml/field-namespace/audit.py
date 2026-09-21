#!/usr/bin/env python3
"""Audit source provenance, single ownership and reproducible Field fixtures."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import fixtures
ROOT=fixtures.ROOT
HERE=fixtures.HERE
sys.path.insert(0,str(ROOT/'tools/ports'))
import inventory

def main():
 checkout=ROOT/'reference_code/rust-osdev/acpi'
 manifest=inventory.read_json(ROOT/'source/libraries/acpi/aml/field-namespace-inventory.json')
 pin=subprocess.check_output(['git','rev-parse','HEAD'],cwd=checkout,text=True).strip()
 assert pin==manifest['upstream']['revision'],pin
 print(json.dumps(inventory.check(manifest,checkout,ROOT),indent=2))
 relocation=json.loads((HERE/'relocation.json').read_text())
 for row in relocation['files']:
  original=subprocess.check_output(['git','show',relocation['base']+':'+row['before']],cwd=ROOT,text=True)
  normalized=original.replace('module '+row['old_module']+';','module '+row['new_module']+';').replace('use aml::','use ').replace('use flags::','use field_flags::').replace('use field_list::','use field_elements::')
  assert hashlib.sha256(original.encode()).hexdigest()==row['original_sha256']
  assert hashlib.sha256(normalized.encode()).hexdigest()==row['relocated_sha256']
  assert (ROOT/row['after']).read_text()==normalized,row['after']
 aml=ROOT/'source/libraries/acpi/aml'
 assert 'depend_as("fields"'not in (aml/'build.omg').read_text()
 assert sum(p.read_text().count('case FieldUnit(')for p in (ROOT/'source').rglob('*.omg'))==1
 for folder in [ROOT/'source/libraries/acpi',ROOT/'tools/ports/acpi']:
  for file in folder.rglob('*'):
   if file.is_file()and file.suffix in ['.omg','.py']:assert ('fields::'+'field_model::')not in file.read_text(),file
 source,selections=fixtures.render(fixtures.cases())
 assert (HERE/'main.omg').read_text()==source
 assert (HERE/'selections.txt').read_text().splitlines()==selections
 assert json.loads((HERE/'cases.json').read_text())==json.loads(json.dumps(fixtures.cases()))
 for file in [aml/'field-namespace.PORT.md',HERE/'README.md']:
  for target in re.findall(r'\]\(([^)]+)\)',file.read_text()):
   if '://'not in target:assert (file.parent/target.split('#')[0]).exists(),(file,target)
 print(f'PASS {len(relocation["files"])} exact parser/metadata moves; one canonical FieldUnit case; {len(fixtures.cases())} reproducible behavior/control pairs.')
 print('Source/provenance audit only; no fresh Omega execution claimed.')
if __name__=='__main__':main()
