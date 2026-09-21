#!/usr/bin/env python3
"""Actual pinned public Interpreter::load_table/evaluate observations."""
import argparse,hashlib,json,os,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;CANONICAL=fixtures.ROOT;UP=CANONICAL/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def observations():
 rows=[]
 for row in fixtures.cases():
  # Public interpreter has no finite-capacity contract. Never ask it to allocate
  # pathological buffer lengths; those belong to Omega preflight tests only.
  if row['name'].startswith('invalid_') or row['name'] in ['maximum_unit','buffer_width32_truncate','buffer_width32_zero','buffer_width64_large','buffer_size_max64','buffer_size_max32','buffer_ones_32','buffer_ones_64']:continue
  if row['kind'] is None and row['name'] not in ['string_missing_nul','string_nonascii','string_nonascii_late','missing_size','buffer_dynamic_arg','buffer_dynamic_local','buffer_dynamic_name','buffer_dynamic_add']:continue
  data=bytes(row['data']);start=row['at'];end=min(row['frame_end'],row['source_length']);literal=data[start:end]
  method=fixtures.package(b'MAIN\0'+b'\xa4'+literal);table=bytes([0x14])+method[1:]
  expected=(row['kind'].lower()+':'+bytes(row['expected']).hex())if row['outcome']=='Success'else None
  rows.append(dict(name=row['name'],table_hex=table.hex(),revision=2 if row['bits']==64 else 1,omega_outcome=row['outcome'],expected=expected))
 # Original UTF8 interpretation differs from AML ASCII encoding.
 method=fixtures.package(b'MAIN\0\xa4\x0d\xc3\xa9\0');rows.append(dict(name='utf8_valid_nonascii',table_hex=(b'\x14'+method[1:]).hex(),revision=2,omega_outcome='BadEncoding',expected=None))
 return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();base=CANONICAL/'tools/ports/acpi/aml-public-execution';binary=Path('/tmp/cathedral-acpi-public-execution/release/cathedral-acpi-public-execution')
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(base/'Cargo.toml'),'--target-dir','/tmp/cathedral-acpi-public-execution'],cwd=CANONICAL,check=True)
 rows=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-byte-literal-public-')as directory:
  for row in observations():
   path=Path(directory)/(row['name']+'.aml');path.write_bytes(bytes.fromhex(row['table_hex']))
   run=subprocess.run([str(binary),str(path),str(row['revision']),''],env=dict(os.environ,CATHEDRAL_GENERIC_DESCRIBE='1'),capture_output=True,text=True,timeout=5)
   assert run.returncode==0,(row['name'],run.stderr)
   observed=dict(line.split('\t',1)for line in run.stdout.replace(str(CANONICAL)+'/','').splitlines());assert observed['forbidden_calls']=='0'and observed['created_mutexes']=='1'
   agreement=row['expected']is not None and observed.get('result')==row['expected'];rows.append({**row,'observed':observed,'agreement':agreement});print(row['name'],observed.get('result',observed.get('panic')),flush=True)
 record=dict(stage='actual public pinned Interpreter::load_table/evaluate; host-only, not an Omega execution claim',upstream_revision=PIN,binary_sha256=sha(binary),generator_sha256=sha(Path(__file__)),fixtures_sha256=sha(HERE/'fixtures.py'),sources={str(x):sha(x)for x in [base/'src/main.rs',base/'Cargo.toml',base/'Cargo.lock']},upstream_sha256={str(x.relative_to(UP)):sha(x)for x in sorted((UP/'src').rglob('*.rs'))},rows=rows)
 if a.write:(HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert record==json.loads((HERE/'reference-verification.json').read_text())
 print(len(rows),'actual public observations',sum(r['agreement']for r in rows),'same-value agreements')
if __name__=='__main__':main()
