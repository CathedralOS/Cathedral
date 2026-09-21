#!/usr/bin/env python3
"""Actual public pinned Interpreter Field reads into inert initialized memory."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import aml_encoding as aml
import vectors
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def bit_length(value):
 for width in range(1,5):
  if value<1<<(6 if width==1 else 4+8*(width-1)):
   return bytes([value])if width==1 else bytes([((width-1)<<6)|(value&15)])+(value>>4).to_bytes(width-1,'little')
 raise ValueError('too many bits')
def table(offset,length,flags,region):
 declaration=b'\x5b\x80REG0\x00'+aml.integer(0x1000)+aml.integer(region)
 fields=(b'\x00'+bit_length(offset)if offset else b'')+b'FLD0'+bit_length(length)
 declaration+=aml.pkg(0x5b81,b'REG0'+bytes([flags])+fields)
 body=b'\xa4FLD0'
 return declaration+aml.method('MAIN',body)
def cases():
 rows=[]
 for vector in vectors.cases():
  if vector['patch'] or vector['offset']>4000 or vector['length']>2048 or vector['name'].startswith(('count_','precedence_')):continue
  rows.append(dict(name=vector['name'],offset=vector['offset'],length=vector['length'],flags=vector['flags'],region=vector['region'],revision=1 if vector['size']==4 else 2,pattern=vector['pattern'],strict_expected=vector['expected']))
 return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');a=p.parse_args();up=a.repository.resolve()/'reference_code/rust-osdev/acpi'
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 source={str(x.relative_to(up)):sha(x)for x in sorted((up/'src').rglob('*.rs'))}
 for relative in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:source[relative]=sha(up/relative)
 for relative,digest in source.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+relative],cwd=up)).hexdigest()==digest
 target=Path('/tmp/cathedral-field-values-public')
 with tempfile.TemporaryDirectory(prefix='field-values-reference-')as d:
  work=Path(d);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-field-values-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n');lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
  binary=target/'release/reference';rows=[]
  for row in cases():
   data=table(row['offset'],row['length'],row['flags'],row['region']);file=work/'table.aml';file.write_bytes(data)
   run=subprocess.run([str(binary),str(file),str(row['revision']),str(row['pattern'])],capture_output=True,text=True,timeout=5,check=True);observed=dict(line.split('\t',1)for line in run.stdout.splitlines());assert observed['forbidden_calls']=='0'
   rows.append(dict(**row,aml_hex=data.hex(),observed=observed))
 record=dict(stage='actual public Interpreter load_table/evaluate with inert Vec callbacks; no hardware, private mirror or Omega native execution',pin=PIN,upstream_sha256=source,binary_sha256=sha(binary),source_sha256={p.name:sha(p)for p in [HERE/'reference.py',HERE/'reference.rs',HERE/'aml_encoding.py',HERE/'vectors.py',HERE/'geometry_vectors.py',lock]},rows=rows)
 path=HERE/'reference-verification.json'
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record
 print('PASS',len(rows),'public field read observations')
if __name__=='__main__':main()
