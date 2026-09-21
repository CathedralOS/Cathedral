#!/usr/bin/env python3
"""Actual public write_buffer_field, using only the real Interpreter-created locked token."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;CANONICAL=Path('/Users/zcanann/Documents/projects/Cathedral');UP=CANONICAL/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def selections():
 rows=[];seen=set()
 for r in fixtures.cases():
  if not r['public']:continue
  payload=r['logical']if r['alias']else r['payload'][:r['payload_length']]
  key=(r['kind'],tuple(r['logical']),r['start'],r['count'],tuple(payload),r['backing_reference'],r['top_reference'])
  if key in seen:continue
  # Never introduce invalid Rust String storage, even to observe its failure.
  if r['kind']=='String':
   original=bytes(r['logical']);original.decode('utf-8')
   if r['count']>0 and r['start']+r['count']<=len(original)*8:
    value=int.from_bytes(original,'little');source=int.from_bytes(payload,'little');mask=((1<<r['count'])-1)<<r['start'];changed=((value&~mask)|((source<<r['start'])&mask)).to_bytes(len(original),'little');changed.decode('utf-8')
  seen.add(key);rows.append(r)
 return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 target=Path('/tmp/cathedral-acpi-field-writes-public');rows=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-field-writes-public-')as directory:
  work=Path(directory);(work/'Cargo.toml').write_text('[package]\nname="cathedral-field-writes-reference"\nversion="0.0.0"\nedition="2024"\n[dependencies]\nacpi={path="'+str(UP)+'"}\n[[bin]]\nname="cathedral-field-writes-reference"\npath="'+str(HERE/'object_reference.rs')+'"\n')
  (work/'Cargo.lock').write_bytes((HERE/'reference.Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--release','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],cwd=CANONICAL,check=True)
  binary=target/'release/cathedral-field-writes-reference'
  for r in selections():
   payload=bytes(r['logical']if r['alias']else r['payload'][:r['payload_length']])
   command=[str(binary),r['kind'],bytes(r['logical']).hex(),str(r['start']),str(r['count']),payload.hex(),r['backing_reference']or'None',r['top_reference']or'None']
   observed=dict(line.split('\t',1)for line in subprocess.check_output(command,text=True,timeout=5).splitlines());assert observed['forbidden_calls']=='0'and observed['created_mutexes']=='1'
   rows.append(dict(case=r,materialized_hex=bytes(r['logical']).hex(),payload_hex=payload.hex(),public_object=observed));print(r['name'],observed,flush=True)
 record=dict(stage='actual public Object::write_buffer_field with Interpreter-created token acquired via its public lock; no fabricated token, private mirror, Store dispatch or firmware GlobalLock',upstream_revision=PIN,binary_sha256=sha(binary),generator_sha256=sha(Path(__file__)),fixtures_sha256=sha(HERE/'fixtures.py'),sources={str(p):sha(p)for p in [HERE/'object_reference.rs',HERE/'reference.Cargo.lock']},upstream_sha256={str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))},rows=rows)
 if a.write:(HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert record==json.loads((HERE/'reference-verification.json').read_text())
 print(len(rows),'actual public field-write observations')
if __name__=='__main__':main()
