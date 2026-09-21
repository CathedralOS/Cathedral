#!/usr/bin/env python3
"""Separate actual public Object methods and explicit public interpreter observations."""
import argparse,hashlib,json,os,re,subprocess,tempfile
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
 rows=[];seen=set()
 for r in fixtures.cases():
  if r['kind']not in ['Integer','String','Buffer','Field','Package','Uninitialized','Reference']:continue
  if r['kind']=='Field'and not r['name'].startswith('field_32_')and not r['name'].startswith('field_64_'):continue
  if r['owned']or r['unit']!=7 or r['object']!=0 or r['object_count']not in [1,2]or r['source_length']!=len(r['data']):continue
  if r['extra']and r['kind']not in ['Field','Reference']:continue
  if any(x==0 or x>=128 for x in r['data'])and r['kind']=='String':continue
  signature=(r['kind'],r['method'],r['bits'],tuple(r['data']),r['number'],r['extra']);
  if signature in seen:continue
  seen.add(signature);rows.append(r)
 return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 base=CANONICAL/'tools/ports/acpi/aml-public-execution';public=Path('/tmp/cathedral-acpi-public-execution/release/cathedral-acpi-public-execution')
 subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(base/'Cargo.toml'),'--target-dir','/tmp/cathedral-acpi-public-execution'],cwd=CANONICAL,check=True)
 target=Path('/tmp/cathedral-acpi-object-conversions-public');observations=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-conversion-public-')as directory:
  work=Path(directory);(work/'Cargo.toml').write_text('[package]\nname="cathedral-object-conversion-reference"\nversion="0.0.0"\nedition="2024"\n[dependencies]\nacpi={path="'+str(UP)+'"}\n[[bin]]\nname="cathedral-object-conversion-reference"\npath="'+str(HERE/'object_reference.rs')+'"\n')
  lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  cmd=['cargo','+nightly-2026-09-04','build','--offline','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)]
  if lock.exists():cmd.append('--locked')
  subprocess.run(cmd,cwd=CANONICAL,check=True)
  if not lock.exists():lock.write_bytes((work/'Cargo.lock').read_bytes())
  binary=target/'release/cathedral-object-conversion-reference'
  for r in selections():
   offset=3 if r['kind']=='Field'else r['number'];width=int(r['name'].rsplit('_',1)[1])if r['kind']=='Field'else 0
   logical=bytes(r['data'])+(bytes(max(0,r['declared']-len(r['data'])))if r['kind']=='Buffer'else b'')
   command=[str(binary),r['kind'],str(r['bits']),logical.hex(),str(offset),str(width),r['method']]
   observed=subprocess.check_output(command,text=True,timeout=5).strip();row=dict(name=r['name'],kind=r['kind'],method=r['method'],bits=r['bits'],data_hex=logical.hex(),source_data_hex=bytes(r['data']).hex(),declared=r['declared'],number=r['number'],field_offset=offset,field_bits=width,omega_error=r['error'],omega_expected=r['expected'],object_method=observed)
   if r['kind']in ['Integer','Buffer','String']:
    val=integer(r['number'])if r['kind']=='Integer'else (b'\x0d'+bytes(r['data'])+b'\0'if r['kind']=='String'else package(0x11,integer(r['declared'])+bytes(r['data'])))
    method=package(0x14,b'MAIN\0\xa4'+bytes([0x99 if r['method']=='integer'else 0x96])+val+b'\0');path=work/'current.aml';path.write_bytes(method)
    out=subprocess.check_output([str(public),str(path),'1'if r['bits']==32 else'2',''],env=dict(os.environ,CATHEDRAL_GENERIC_DESCRIBE='1'),text=True,timeout=5)
    parsed=dict(line.split('\t',1)for line in out.splitlines());assert parsed['forbidden_calls']=='0'and parsed['created_mutexes']=='1';row.update(table_hex=method.hex(),explicit_interpreter=parsed)
   observations.append(row);print(r['name'],observed,flush=True)
 record=dict(stage='actual public Object methods and distinct public explicit Interpreter evaluations; no private mirrors',upstream_revision=PIN,object_binary_sha256=sha(binary),interpreter_binary_sha256=sha(public),generator_sha256=sha(Path(__file__)),fixtures_sha256=sha(HERE/'fixtures.py'),sources={str(p):sha(p)for p in [HERE/'object_reference.rs',HERE/'reference.Cargo.lock',base/'src/main.rs',base/'Cargo.toml',base/'Cargo.lock']},upstream_sha256={str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))},rows=observations)
 if a.write:(HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert record==json.loads((HERE/'reference-verification.json').read_text())
 print(len(observations),'Object observations,',sum('explicit_interpreter'in r for r in observations),'explicit interpreter observations')
if __name__=='__main__':main()
