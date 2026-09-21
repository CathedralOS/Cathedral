#!/usr/bin/env python3
"""Actual public pinned Object::read_buffer_field observations, including shape differences."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;CANONICAL=fixtures.ROOT;UP=CANONICAL/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def selections():
 rows=[];seen=set()
 for r in fixtures.cases():
  if not r['public']:continue
  logical=bytes(r['data'])+b'\0'*max(0,r['declared']-len(r['data']))
  key=(r['bits'],r['kind'],logical,r['start'],r['count'],r['backing_reference'],r['top_reference'])
  if key in seen:continue
  seen.add(key);rows.append(r)
 return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 target=Path('/tmp/cathedral-acpi-field-values-public');rows=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-field-values-public-')as directory:
  work=Path(directory);(work/'Cargo.toml').write_text('[package]\nname="cathedral-field-values-reference"\nversion="0.0.0"\nedition="2024"\n[dependencies]\nacpi={path="'+str(UP)+'"}\n[[bin]]\nname="cathedral-field-values-reference"\npath="'+str(HERE/'object_reference.rs')+'"\n')
  (work/'Cargo.lock').write_bytes((HERE/'reference.Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--release','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],cwd=CANONICAL,check=True)
  binary=target/'release/cathedral-field-values-reference'
  for r in selections():
   logical=bytes(r['data'])+b'\0'*max(0,r['declared']-len(r['data']))
   command=[str(binary),str(r['bits']),r['kind'],logical.hex(),str(r['start']),str(r['count']),r['backing_reference']or'None',r['top_reference']or'None']
   observed=subprocess.check_output(command,text=True,timeout=5).strip()
   expected='failure:'+r['error']if r['error']else 'integer:'+str(r['expected_integer'])if r['expected_integer']is not None else 'buffer:'+bytes(r['expected_buffer']).hex()
   rows.append(dict(case=r,materialized_hex=logical.hex(),omega_expected=expected,public_object=observed));print(r['name'],observed,flush=True)
 record=dict(stage='actual pinned public Object::read_buffer_field; no mirror or ObjectToken',upstream_revision=PIN,binary_sha256=sha(binary),generator_sha256=sha(Path(__file__)),fixtures_sha256=sha(HERE/'fixtures.py'),sources={str(p):sha(p)for p in [HERE/'object_reference.rs',HERE/'reference.Cargo.lock']},upstream_sha256={str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))},rows=rows)
 if a.write:(HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert record==json.loads((HERE/'reference-verification.json').read_text())
 print(len(rows),'actual public field-read observations')
if __name__=='__main__':main()
