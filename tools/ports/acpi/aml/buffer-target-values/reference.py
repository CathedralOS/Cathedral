#!/usr/bin/env python3
"""Actual pinned public Object::replace_with_implicit_casting on local values."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cases():
 rows=[]
 def add(name,kind,data,extent,bits=64):
  admitted=extent>0 and(kind=='Integer'or(kind=='String'and len(data)>0))
  raw=data.to_bytes(bits//8,'little')if kind=='Integer'else data
  prepared=(raw[:extent]+bytes(max(0,extent-len(raw))))if admitted else None
  rows.append(dict(name=name,kind=kind,source=str(data)if kind=='Integer'else data.hex(),extent=extent,bits=bits,profile_admitted=admitted,prepared_hex=prepared.hex()if prepared is not None else None))
 for bits in [32,64]:
  for number in [0,0x8877665544332211]:
   for extent in [0,1,4,8,256]:add(f'integer_{bits}_{number}_{extent}','Integer',number&((1<<bits)-1),extent,bits)
 for length in [0,1,3,255,256]:
  data=bytes(65+i%26 for i in range(length))
  for extent in [0,1,3,8,256]:add(f'string_{length}_{extent}','String',data,extent)
 for length in [0,2,256]:
  for extent in [0,1,4]:add(f'buffer_{length}_{extent}','Buffer',bytes(i%256 for i in range(length)),extent)
 return rows

def upstream(repository):
 up=repository.resolve()/'reference_code/rust-osdev/acpi';assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 source={str(p.relative_to(up)):sha(p)for p in sorted((up/'src').rglob('*.rs'))}
 for n in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:source[n]=sha(up/n)
 for n,h in source.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+n],cwd=up)).hexdigest()==h
 return up,source

def classify(row,observed):
 raw=int(row['source']).to_bytes(8,'little')if row['kind']=='Integer'else bytes.fromhex(row['source'])
 assert observed['result']==f'buffer:{len(raw)}:{raw.hex()}'
 if not row['profile_admitted']:return 'excluded-profile-observation'
 return 'matches'if raw.hex()==row['prepared_hex']else'differs'
def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');a=p.parse_args();up,source=upstream(a.repository);target=Path('/tmp/cathedral-buffer-target-values-public')
 with tempfile.TemporaryDirectory(prefix='buffer-target-values-public-')as d:
  work=Path(d);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-buffer-target-values-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n');lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
  binary=target/'release/reference';rows=[]
  for row in cases():
   run=subprocess.run([str(binary),row['kind'],row['source'],str(row['extent'])],capture_output=True,text=True,timeout=5,check=True);observed=dict(line.split('\t',1)for line in run.stdout.splitlines());classification=classify(row,observed);rows.append(dict(**row,observed=observed,classification=classification))
 record=dict(stage='actual public Object::replace_with_implicit_casting on ordinary local owned Rust values; no interpreter, private mirror, ObjectToken or services',pin=PIN,upstream_sha256=source,binary_sha256=sha(binary),source_sha256={p.name:sha(p)for p in [HERE/'reference.py',HERE/'reference.rs',lock]},rows=rows)
 path=HERE/'reference-verification.json'
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record
 print('PASS',len(rows),'public observations;',dict((k,sum(r['classification']==k for r in rows))for k in ['matches','differs','excluded-profile-observation']))
if __name__=='__main__':main()
