#!/usr/bin/env python3
"""Actual public Interpreter evaluations; no private formatting mirror."""
import argparse,hashlib,json,os,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;CANONICAL=Path('/Users/zcanann/Documents/projects/Cathedral');UP=CANONICAL/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def integer(n):
 if n in [0,1]:return bytes([n])
 w=next(w for w in [1,2,4,8]if n<1<<(w*8));return bytes([{1:10,2:11,4:12,8:14}[w]])+n.to_bytes(w,'little')
def package(op,body):
 w=next(w for w in range(1,5)if len(body)+w<(64 if w==1 else 1<<(4+8*(w-1))));n=len(body)+w
 return bytes([op])+(bytes([n])if w==1 else bytes([((w-1)<<6)|(n&15)])+(n>>4).to_bytes(w-1,'little'))+body

def selections():
 rows=[]
 for r in fixtures.cases():
  if r['kind']not in ['Integer','String','Buffer']:continue
  if r['owned']or r['unit']!=7 or r['object']!=0 or r['object_count']!=1 or r['source_length']!=len(r['data'])or r['extra']:continue
  if r['kind']=='String'and any(x==0 or x>=128 for x in r['data']):continue
  if r['declared']>256:continue
  rows.append(r)
 return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 base=CANONICAL/'tools/ports/acpi/aml-public-execution';target=Path('/tmp/cathedral-acpi-numeric-string-public')
 subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(base/'Cargo.toml'),'--target-dir',str(target)],cwd=CANONICAL,check=True)
 binary=target/'release/cathedral-acpi-public-execution';observations=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-numeric-string-public-')as directory:
  work=Path(directory)
  for r in selections():
   val=integer(r['number'])if r['kind']=='Integer'else(b'\x0d'+bytes(r['data'])+b'\0'if r['kind']=='String'else package(0x11,integer(r['declared'])+bytes(r['data'])))
   method=package(0x14,b'MAIN\0\xa4'+bytes([0x97 if r['format']=='Decimal'else 0x98])+val+b'\0');path=work/'current.aml';path.write_bytes(method)
   run=subprocess.run([str(binary),str(path),'1'if r['bits']==32 else'2',''],env=dict(os.environ,CATHEDRAL_GENERIC_DESCRIBE='1'),text=True,capture_output=True,timeout=5)
   parsed=dict(line.split('\t',1)for line in run.stdout.splitlines());assert run.returncode==0 and parsed['forbidden_calls']=='0'and parsed['created_mutexes']=='1',(r['name'],run.stdout,run.stderr)
   observations.append(dict(case=r,table_hex=method.hex(),public_interpreter=parsed,stderr=run.stderr));print(r['name'],parsed.get('result'),flush=True)
 record=dict(stage='actual public pinned Interpreter ToDecimalString/ToHexString synthetic opcode evaluations; no private mirrors',upstream_revision=PIN,interpreter_binary_sha256=sha(binary),generator_sha256=sha(Path(__file__)),fixtures_sha256=sha(HERE/'fixtures.py'),sources={str(p):sha(p)for p in[base/'src/main.rs',base/'Cargo.toml',base/'Cargo.lock']},upstream_sha256={str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))},rows=observations)
 if a.write:(HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert record==json.loads((HERE/'reference-verification.json').read_text())
 print(len(observations),'actual public opcode observations')
if __name__=='__main__':main()
