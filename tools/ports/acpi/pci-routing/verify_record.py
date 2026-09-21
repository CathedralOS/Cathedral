#!/usr/bin/env python3
"""Validate retained public observations without executing Rust or Omega."""
import collections,hashlib,json,subprocess
import check,fixtures

def main():
 subprocess.run(['python3',str(check.HERE/'fixtures.py'),'--check'],check=True,cwd=check.ROOT)
 record=json.loads((check.HERE/'observations.json').read_text())
 assert record['upstream_revision']==check.PIN
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=check.UP,text=True).strip()==check.PIN
 assert record['source_sha256']==check.inputs(),'harness closure drift'
 upstream={str(p.relative_to(check.UP)):check.sha(p)for p in sorted((check.UP/'src').rglob('*.rs'))}
 assert record['upstream_source_sha256']==upstream,'upstream source drift'
 assert record['binary_sha256']==check.sha(check.TARGET/'release/cathedral-acpi-pci-routing')
 cases=fixtures.cases();assert set(record['rows'])=={r['name']for r in cases};counts=collections.Counter()
 for row in cases:
  observation=record['rows'][row['name']];values=observation['observed']
  assert observation['table_sha256']==hashlib.sha256(bytes.fromhex(row['aml_hex'])).hexdigest()
  assert observation['strict_expectation']==row['strict_expectation']
  assert values['forbidden_calls']=='0'and values['created_mutexes']=='1'
  counts['load_'+('success'if values.get('load')=='ok'else'panic'if values.get('load_or_probe')=='panic'else'error')]+=1
  decode=values.get('decode','not reached')
  category='not_reached'if 'decode'not in values else'success'if decode.startswith('ok:')else'panic'if decode=='panic'else'error'
  counts['decode_'+category]+=1
  routes={key:check.irq(value)for key,value in values.items()if key.startswith('route:')}
  assert routes==observation['normalized_routes']
  assert len(routes)==(len(row['queries'])if category=='success'else 0)
  for result in routes.values():counts['route_'+('success'if result['outcome']=='success'else'panic'if result['outcome']=='panic'else'error')]+=1
 assert dict(counts)==record['counts']
 print('PASS',len(cases),'public fixture records;',len(record['source_sha256']),'harness hashes;',len(upstream),'upstream Rust hashes;',json.dumps(dict(counts),sort_keys=True))
if __name__=='__main__':main()
