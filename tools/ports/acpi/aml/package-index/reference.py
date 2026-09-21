#!/usr/bin/env python3
"""Actual pinned public Interpreter Package Index probes; all services trapped."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import aml_encoding as aml
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cases():
 rows=[]
 def add(label,values,declared=None):
  count=len(values)if declared is None else declared
  for index in sorted(set([0,max(0,count-1),count,(1<<64)-1]+(list(range(count))if count<=5 else[]))):
   rows.append(dict(name=label+'_'+str(index),values=values,count=count,index=index,success=index<count))
 add('empty',[]);add('single',[11]);add('mixed',[11,{'text':'ABC'},{'buffer':'ff00'}, {'package':[99]}, {'name':'LATE'}]);add('uninitialized',[],1);add('long',list(range(63)))
 return rows
def table(row):
 return b'\x08'+aml.name('PKG')+aml.pkg(0x12,bytes([row['count']])+b''.join(map(aml.value,row['values'])))+aml.method('MAIN',b'\xa4\x88'+aml.name('PKG')+aml.integer(row['index'])+b'\x00')
def validate(row,observed):
 assert observed['load']=='ok' and observed['forbidden_calls']=='0' and observed['created_mutexes']=='1' and 'panic'not in observed,observed
 if row['success']:
  assert observed['kind']==observed['second_kind']=='RefOf' and observed['same_element']==observed['fresh_wrappers']=='true',observed
  name=row['index']<len(row['values'])and isinstance(row['values'][row['index']],dict)and'name'in row['values'][row['index']]
  assert observed['retained_name_path']==str(name).lower(),observed
 else:assert observed['error']=='IndexOutOfBounds|IndexOutOfBounds',observed
def upstream(repository):
 up=repository.resolve()/'reference_code/rust-osdev/acpi';assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 source={str(p.relative_to(up)):sha(p)for p in sorted((up/'src').rglob('*.rs'))}
 for n in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:source[n]=sha(up/n)
 for n,h in source.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+n],cwd=up)).hexdigest()==h
 return up,source
def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');a=p.parse_args();up,source=upstream(a.repository)
 target=Path('/tmp/cathedral-package-index-public')
 with tempfile.TemporaryDirectory(prefix='package-index-public-')as d:
  work=Path(d);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-package-index-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n');lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
  binary=target/'release/reference';rows=[]
  for row in cases():
   data=table(row);file=work/'table.aml';file.write_bytes(data)
   r=subprocess.run([str(binary),str(file),str(row['index'])],capture_output=True,text=True,timeout=5,check=True);observed=dict(line.split('\t',1)for line in r.stdout.splitlines());validate(row,observed)
   rows.append(dict(**row,aml_hex=data.hex(),observed=observed))
 record=dict(stage='actual public Interpreter::new/load_table/evaluate, synthetic AML, all hardware and services trapped; no private mirrors',pin=PIN,upstream_sha256=source,binary_sha256=sha(binary),source_sha256={p.name:sha(p)for p in [HERE/'reference.py',HERE/'reference.rs',HERE/'aml_encoding.py',lock]},rows=rows)
 path=HERE/'reference-verification.json'
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record
 print('PASS',len(rows),'public package Index observations; actual RefOf identity, fresh wrappers and error outcomes')
if __name__=='__main__':main()
