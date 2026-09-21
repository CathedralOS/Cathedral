#!/usr/bin/env python3
"""Actual public pinned Interpreter named Store; independent primary-policy oracle."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import aml_encoding as aml
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
MAPPING=['source/libraries/acpi/aml/namespace.omg','source/libraries/acpi/aml/object_references.omg','source/libraries/acpi/aml/byte_storage.omg']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cases():
 rows=[]
 for kind in ['Buffer','String']:
  for size in [0,1,8,255,256]:
   for index in sorted({0,max(0,size-1),size,1<<63,(1<<64)-1}):
    rows.append(dict(name=f'{kind}_{size}_{index}',kind=kind,size=size,index=index,success=index<size))
 return rows

def table(row):
 data=bytes([65])*row['size']
 value={'buffer':data.hex()}if row['kind']=='Buffer'else{'text':data.decode()}
 return aml.named('BACK',value)+aml.method('MAIN',b'\xa4\x88'+aml.name('BACK')+aml.integer(row['index'])+b'\x00')
def validate(row,observed):
 assert observed['load']=='ok'and observed['forbidden_calls']=='0'and observed['created_mutexes']=='1'and'panic'not in observed,observed
 if row['success']:
  assert observed['kind']==observed['second_kind']=='RefOf'and observed['same_backing']==observed['fresh_wrappers']==observed['fresh_fields']=='true',observed
  assert observed['geometry']==f"{row['index']*8}:8|{row['index']*8}:8",observed
 else:assert observed['error']=='IndexOutOfBounds|IndexOutOfBounds',observed

def upstream(repository):
 up=repository.resolve()/'reference_code/rust-osdev/acpi';assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 source={str(p.relative_to(up)):sha(p)for p in sorted((up/'src').rglob('*.rs'))}
 for n in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:source[n]=sha(up/n)
 for n,h in source.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+n],cwd=up)).hexdigest()==h
 return up,source

def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');p.add_argument('--smoke',action='store_true');a=p.parse_args();up,source=upstream(a.repository);mapping={n:sha(ROOT/n)for n in MAPPING};inputs={n:sha(HERE/n)for n in ['reference.py','reference.rs','aml_encoding.py']}
 target=Path('/tmp/cathedral-byte-index-public')
 with tempfile.TemporaryDirectory(prefix='byte-index-public-')as d:
  work=Path(d);manifest=f'[package]\nname="cathedral-byte-index-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n';(work/'Cargo.toml').write_text(manifest);lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  inputs['reference.Cargo.lock']=sha(lock)
  subprocess.run(['cargo','+nightly-2026-09-04','build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
  binary=target/'release/reference';rows=[];selected=cases()
  if a.smoke:selected=[selected[i]for i in [0,4,9,16,19]]
  for row in selected:
   data=table(row);file=work/'table.aml';file.write_bytes(data)
   r=subprocess.run([str(binary),str(file)],capture_output=True,text=True,timeout=5,check=True);observed=dict(line.split('\t',1)for line in r.stdout.splitlines());validate(row,observed);rows.append(dict(**row,aml_hex=data.hex(),observed=observed))
 assert mapping=={n:sha(ROOT/n)for n in MAPPING}and inputs=={n:sha(HERE/n)for n in inputs}
 record=dict(execution_root=str(ROOT.resolve()),probe_root=str(HERE),build_text=manifest,build_sha256=hashlib.sha256(manifest.encode()).hexdigest(),production_mapping_sha256=mapping,notes=['Two actual calls per row; repeated construction retains the same backing identity and creates distinct RefOf and BufferField identities.','No target Store or field read/write is claimed; method Index uses NullName destination.'],stage='actual public Interpreter::new/load_table/evaluate, synthetic AML, all services trapped; no private mirrors',pin=PIN,upstream_sha256=source,binary_sha256=sha(binary),source_sha256={n:sha(HERE/n)for n in inputs},rows=rows)
 path=HERE/('reference-smoke.json'if a.smoke else'reference-verification.json')
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record
 print('PASS',len(rows),'public byte Index observations;', {'success':sum(r['success']for r in rows),'error':sum(not r['success']for r in rows)})
if __name__=='__main__':main()
