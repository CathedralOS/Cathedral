#!/usr/bin/env python3
"""Actual public pinned Interpreter Field writes into inert initialized memory."""
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
def table(offset,length,flags,region,source):
 declaration=b'\x5b\x80REG0\x00'+aml.integer(0x1000)+aml.integer(region)
 fields=(b'\x00'+bit_length(offset)if offset else b'')+b'FLD0'+bit_length(length)
 declaration+=aml.pkg(0x5b81,b'REG0'+bytes([flags])+fields)
 body=b'\x70'+aml.value(source)+b'FLD0\xa4\x00'
 return declaration+aml.method('MAIN',body)
def cases():
 rows=[]
 for row in vectors.cases():
  if row['ordinal']!=0 or row['owned'] or row['extra'] or row['name'].startswith(('padding','initializer','declared','integer_unused')) or 'error'in row['expected']:continue
  kind=row['kind']
  if kind=='Integer':
   if row['width']not in [1,64,2048]:continue
   source=row['number']&((1<<row['bits'])-1)
  elif kind=='Buffer':source=dict(buffer=bytes(row['data']).hex())
  elif kind=='String':source=dict(text=bytes(row['data']).decode('ascii'))
  else:continue
  total=row['expected']['total'];primary=[]
  for ordinal in range(total):
   expected=vectors.expected(dict(row,ordinal=ordinal));primary.append(dict(ordinal=ordinal,length=expected['length'],hex=bytes(expected['bytes'][:expected['length']]).hex()))
  rows.append(dict(name=row['name'],offset=3,length=row['width'],flags=1,region=300,revision=1 if row['bits']==32 else 2,source=source,primary_total=total,primary=primary))
 return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');a=p.parse_args();up=a.repository.resolve()/'reference_code/rust-osdev/acpi'
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 source={str(x.relative_to(up)):sha(x)for x in sorted((up/'src').rglob('*.rs'))}
 for relative in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:source[relative]=sha(up/relative)
 for relative,digest in source.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+relative],cwd=up)).hexdigest()==digest
 target=Path('/tmp/cathedral-field-sources-public')
 with tempfile.TemporaryDirectory(prefix='field-sources-reference-')as d:
  work=Path(d);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-field-sources-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n');lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
  binary=target/'release/reference';rows=[]
  for row in cases():
   data=table(row['offset'],row['length'],row['flags'],row['region'],row['source']);file=work/'table.aml';file.write_bytes(data)
   run=subprocess.run([str(binary),str(file),str(row['revision'])],capture_output=True,text=True,timeout=5,check=True);observed=dict(line.split('\t',1)for line in run.stdout.splitlines());assert observed['forbidden_calls']=='0'
   rows.append(dict(**row,aml_hex=data.hex(),observed=observed))
 record=dict(stage='actual public Interpreter load_table/evaluate with inert Vec callbacks; no hardware, private mirror or Omega native execution',pin=PIN,upstream_sha256=source,binary_sha256=sha(binary),source_sha256={p.name:sha(p)for p in [HERE/'reference.py',HERE/'reference.rs',HERE/'aml_encoding.py',HERE/'vectors.py',HERE/'geometry_vectors.py',lock]},rows=rows)
 path=HERE/'reference-verification.json'
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record
 print('PASS',len(rows),'public field source observations')
if __name__=='__main__':main()
