#!/usr/bin/env python3
"""Record public pinned AmlName/NameSeg text operations, including caught panics."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cases():
 rows=[]
 for kind in ['path','segment']:
  values=['','A','AB','ABC','ABCD','ABCDE','_','A0','A_9Z','0','0ABC','a','ab','abc','abcd','aBcD','!','A!','AB!','ABC!',' A','A ','A\0','A\n','é','Aé','ABCé']
  if kind=='path':values+=['\\','^','^^','^A','^^A','\\A','\\_SB.PCI0','A.B','A.B.C','A.B.C.D','.','A.','.A','A..B','\\.A','\\^A','A.^B','A.^^B','A.\\B','^^.A','\\\\A','A\\B','^'*16,'^'*17,'^'*16+'ABCD','^'*17+'ABCD','.'.join(['ABCD']*16),'.'.join(['ABCD']*17),'\\'+'.'.join(['ABCD']*16),'^'*16+'.'.join(['ABCD']*16)]
  rows += [dict(kind=kind,text_hex=text.encode().hex())for text in values]
 return rows
def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');a=p.parse_args();repo=a.repository.resolve();up=repo/'reference_code/rust-osdev/acpi'
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 sources={str(x.relative_to(up)):sha(x)for x in sorted((up/'src').rglob('*.rs'))}
 for rel in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:sources[rel]=sha(up/rel)
 for rel,digest in sources.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+rel],cwd=up)).hexdigest()==digest
 with tempfile.TemporaryDirectory(prefix='cathedral-name-text-reference-')as d:
  work=Path(d);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-name-text-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n');lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-name-text-public-reference'],check=True)
 binary=Path('/tmp/cathedral-name-text-public-reference/release/reference');observations=[]
 for row in cases():
  run=subprocess.run([str(binary),row['kind'],row['text_hex']],capture_output=True,text=True,timeout=5,check=True);value=dict(line.split('\t',1)for line in run.stdout.splitlines());observations.append(dict(**row,observed=value))
 data=dict(stage='actual pinned public AmlName/NameSeg operations; no AML evaluation, native Omega or hardware',pin=PIN,upstream_sha256=sources,binary_sha256=sha(binary),source_sha256={p.name:sha(p)for p in [HERE/'reference.py',HERE/'reference.rs',lock]},observations=observations)
 path=HERE/'reference-verification.json'
 if a.write:path.write_text(json.dumps(data,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==data
 print('PASS',len(observations),'public name observations;',sum('panic'in x['observed']for x in observations),'caught pin panics')
if __name__=='__main__':main()
