#!/usr/bin/env python3
"""Actual pinned public Interpreter ToString probes; all services trapped."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import aml_encoding as aml
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cases():
 rows=[]
 patterns=[b'',b'ABC',b'\0ABC',b'A\0\xff',b'AB\0C',b'A\xffC',b'\xc3\xa9',b'\xc3\xa9\0',b'\x7f',b'A'*255+b'\0',b'Z'*256]
 for i,data in enumerate(patterns):
  for maximum in [0,1,2,(1<<64)-1]:
   prefix=data[:maximum].split(b'\0')[0];strict={'error':'Encoding'}if any(b>127 for b in prefix)else{'hex':prefix.hex()}
   rows.append(dict(name=f'public_{i}_{maximum}',data=data.hex(),maximum=maximum,strict=strict))
 return rows
def upstream(repository):
 up=repository.resolve()/'reference_code/rust-osdev/acpi';assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 source={str(p.relative_to(up)):sha(p)for p in sorted((up/'src').rglob('*.rs'))}
 for n in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:source[n]=sha(up/n)
 for n,h in source.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+n],cwd=up)).hexdigest()==h
 return up,source
def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');a=p.parse_args();up,source=upstream(a.repository)
 target=Path('/tmp/cathedral-object-to-string-public')
 with tempfile.TemporaryDirectory(prefix='object-to-string-public-')as d:
  work=Path(d);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-object-to-string-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n');lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
  binary=target/'release/reference';rows=[]
  for row in cases():
   data=aml.method('MAIN',b'\xa4\x9c'+aml.value(dict(buffer=row['data']))+aml.integer(row['maximum'])+b'\x00');file=work/'table.aml';file.write_bytes(data)
   r=subprocess.run([str(binary),str(file),'2',''],capture_output=True,text=True,timeout=5,check=True);observed=dict(line.split('\t',1)for line in r.stdout.splitlines());assert observed['forbidden_calls']=='0'and observed['load']=='ok',observed
   # This predicts the narrow pinned branch for audit only, never Omega's oracle.
   buf=bytes.fromhex(row['data']);nul=buf.find(b'\0');raw=(buf if nul<0 else buf[:nul+1])[:row['maximum']]
   try:raw.decode('utf8');assert observed.get('result')=='string:'+raw.hex(),observed
   except UnicodeDecodeError:assert observed.get('result','').startswith('error:InvalidOperationOnObject'),observed
   matches=observed.get('result')=='string:'+row['strict'].get('hex','!')or('error'in row['strict']and observed.get('result','').startswith('error:'))
   rows.append(dict(**row,aml_hex=data.hex(),observed=observed,matches_strict=matches))
 record=dict(stage='actual public Interpreter::new/load_table/evaluate, synthetic AML, all hardware and services trapped; no private mirrors',pin=PIN,upstream_sha256=source,binary_sha256=sha(binary),source_sha256={p.name:sha(p)for p in [HERE/'reference.py',HERE/'reference.rs',HERE/'aml_encoding.py',lock]},rows=rows)
 path=HERE/'reference-verification.json'
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record
 print('PASS',len(rows),'public observations;',sum(r['matches_strict']for r in rows),'matching,',sum(not r['matches_strict']for r in rows),'raw differences')
if __name__=='__main__':main()
