#!/usr/bin/env python3
"""Verify exact compiled Field fixture inputs/results without executing Omega."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import check
import fixtures

def digest(text):return hashlib.sha256(text.encode()).hexdigest()
def recorded_build(record,current):
 original=current.replace(str(fixtures.ROOT),record['execution_root'])
 assert record['build_source']==original and record['build_sha256']==digest(original)
def passes(output,names):
 found=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',output,re.M)
 assert len(found)==len(names)
 assert [name+'='+expected for name,expected,actual in found if expected==actual]==names

def migration(file):
 spec=importlib.util.spec_from_file_location('field_migration',fixtures.HERE.parent/'fields/migration.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
 record=json.loads(file.read_text());assert record['source_unchanged']and record['exit_code']==0
 for path,sha in record['source_sha256'].items():assert check.sha(fixtures.ROOT/path)==sha,path
 # New unused sibling packages do not alter this retained dependency closure.
 added=set(module.snapshot())-set(record['source_sha256'])
 assert all(path.startswith('source/libraries/acpi/field_protocol/')for path in added),added
 assert '/field_protocol'not in record['build_source']
 source,names=module.fixture();assert record['fixture_sha256']==digest(source)and record['selections']==names
 # Earlier migration receipts predate the explicit execution_root field.
 original_root=str(Path(record['build_source'].split('location:"',1)[1].split('"',1)[0]).parents[3])
 expected=module.build().replace(str(fixtures.ROOT),original_root)
 assert record['build_source']==expected and record['build_sha256']==digest(expected)
 assert record['binary_sha256']==check.sha(record['binary']);passes(record['output'],names)
 print('PASS retained migration inputs/results; unused new sibling files:',len(added))

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('records',nargs='*',type=Path);p.add_argument('--migration',type=Path);a=p.parse_args();assert a.records or a.migration
 if a.migration:migration(a.migration)
 for file in a.records:
  record=json.loads(file.read_text());assert record['source_unchanged']
  current=check.snapshot()
  if record['stage']=='checked interpreter':assert record['source_sha256']==current,file
  else:
   # Actual Omega inputs are production sources and the compiled selected pair.
   # Auxiliary generator snapshots remain recorded historically if an unrelated
   # scenario or a receipt checker changed after this completed constant run.
   production={k:v for k,v in record['source_sha256'].items()if k.startswith('source/')}
   assert production=={k:v for k,v in current.items()if k.startswith('source/')},file
  assert record['binary_sha256']==check.sha(record['binary']);recorded_build(record,check.build())
  rows=[r for name in record['cases']for r in fixtures.cases()if r['name']==name]
  assert len(rows)==len(record['cases'])==record['scenarios']==record['controls']
  if record['stage']=='checked interpreter':
   source,names=fixtures.render(rows);assert len(record['results'])==1
   result=record['results'][0];assert result['exit_code']==0 and result['fixture_sha256']==digest(source);passes(result['output'],names)
  else:
   assert record['stage']=='constant evaluator'and len(rows)==1 and len(record['results'])==2
   for i,result in enumerate(record['results']):
    source=fixtures.IMPORTS+fixtures.HELPERS+'machine test()->i32{'+fixtures.fixture_body(rows[0],bool(i))+'}\nconst RESULT:i32=test();\nmachine require_ok(value:i32) requires value==0; {}\ndata Main {}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
    assert result['fixture_sha256']==digest(source)
    if i==0:assert result['exit_code']==0
    else:assert result['exit_code']!=0 and 'cannot prove requires contract'in result['output']and ('1 == 0'in result['output']or'1==0'in result['output'])
   print('Constant compiled inputs and production hashes match; entire auxiliary generator snapshot current:',record['source_sha256']==current)
  print('PASS retained input/results:',file)
 print('Record verification only; no fresh compiler, native or hardware execution.')
if __name__=='__main__':main()
