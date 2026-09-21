#!/usr/bin/env python3
"""Actual public pinned Interpreter Concatenate observations, never private mirrors."""
import argparse,hashlib,json,os,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;CANONICAL=fixtures.ROOT;UP=CANONICAL/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def integer(n):
 if n in [0,1]:return bytes([n])
 w=next(w for w in [1,2,4,8]if n<1<<(w*8));return bytes([{1:10,2:11,4:12,8:14}[w]])+n.to_bytes(w,'little')
def package(op,body):
 w=next(w for w in range(1,5)if len(body)+w<(64 if w==1 else 1<<(4+8*(w-1))));n=len(body)+w
 return bytes([op])+(bytes([n])if w==1 else bytes([((w-1)<<6)|(n&15)])+(n>>4).to_bytes(w-1,'little'))+body

def selections():
 rows=[]
 samples=[fixtures.node('Integer',number=n)for n in [0,12,fixtures.MAX]]+[fixtures.node('String',s)for s in [b'',b'S']]+[fixtures.node('Buffer',s)for s in [b'',b'A',b'\0']]
 for bits in [32,64]:
  for kind in ['Package','Device']:
   for i,n in enumerate(samples):rows.append(dict(name=f'{kind}_basic_{bits}_{i}',a=fixtures.node(kind),b=n,bits=bits))
   for n in [fixtures.node('Integer',number=12),fixtures.node('String',b'S'),fixtures.node('Buffer',b'A')]:rows.append(dict(name=f'{n["kind"]}_{kind}_{bits}',a=n,b=fixtures.node(kind),bits=bits))
   for other in ['Package','Device']:rows.append(dict(name=f'{kind}_{other}_{bits}',a=fixtures.node(kind),b=fixtures.node(other),bits=bits))
 return rows

def term(n):
 k=n['kind'];b=bytes(n['data'])
 if k=='Integer':return integer(n['number'])
 if k=='String':return b'\x0d'+b+b'\0'
 if k=='Buffer':return package(0x11,integer(len(b))+b)
 if k=='Package':return package(0x12,b'\0')
 assert k=='Device';return b'DEV0'
def aml(r):
 prefix=b'\x5b'+package(0x82,b'DEV0')if any(n['kind']=='Device'for n in [r['a'],r['b']])else b''
 return prefix+package(0x14,b'MAIN\0\xa4\x73'+term(r['a'])+term(r['b'])+b'\0')
def expected(r):
 a,b=r['a'],r['b'];ka,kb=a['kind'],b['kind'];bits=r['bits']
 if ka in ['Integer','Buffer']:return dict(kind='error',reason='ToInteger'if ka=='Integer'else'ConvertToBuffer')
 def pin_text(n):
  k=n['kind']
  if k in ['Package','Device']:return fixtures.LABELS[k].encode()
  if k=='Integer':return str(n['number']).encode()
  return bytes(n['data'])
 return dict(kind='string',bytes=(pin_text(a)+pin_text(b)).hex())
def verify():
 record=json.loads((HERE/'reference-verification.json').read_text());assert record['generator_sha256']==sha(Path(__file__))and record['fixtures_sha256']==sha(HERE/'fixtures.py')
 assert record['upstream_revision']==PIN and record['execution_root']==str(ROOT.resolve())
 assert record['interpreter_binary_sha256']==sha(Path(record['interpreter_binary']))
 for p,h in record['sources'].items():assert sha(Path(p))==h
 for p,h in record['upstream_sha256'].items():assert sha(UP/p)==h
 assert len(record['rows'])==len(selections())
 for observation,r in zip(record['rows'],selections()):
  assert observation['case']==r and observation['table_hex']==aml(r).hex()and observation['expected_pinned']==expected(r)
  parsed=observation['public_interpreter'];assert parsed['forbidden_calls']=='0'and parsed['created_mutexes']=='1'
  exp=expected(r)
  if exp['kind']=='string':assert parsed['result']=='string:'+exp['bytes'],(r['name'],parsed)
  else:assert parsed['result'].startswith('error:')and exp['reason']in parsed['result'],(r['name'],parsed)
 print(len(record['rows']),'actual public pinned Concatenate observations verified')
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args()
 if a.check:verify();return
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 base=CANONICAL/'tools/ports/acpi/aml-public-execution';target=Path('/tmp/cathedral-acpi-concat-described-public')
 subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(base/'Cargo.toml'),'--target-dir',str(target)],cwd=CANONICAL,check=True)
 binary=target/'release/cathedral-acpi-public-execution';observations=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-concat-described-public-')as directory:
  path=Path(directory)/'current.aml'
  for r in selections():
   table=aml(r);path.write_bytes(table)
   run=subprocess.run([str(binary),str(path),'1'if r['bits']==32 else'2',''],env=dict(os.environ,CATHEDRAL_GENERIC_DESCRIBE='1'),text=True,capture_output=True,timeout=5)
   parsed=dict(line.split('\t',1)for line in run.stdout.splitlines());assert run.returncode==0 and parsed['forbidden_calls']=='0'and parsed['created_mutexes']=='1',(r['name'],run.stdout,run.stderr)
   observations.append(dict(case=r,table_hex=table.hex(),expected_pinned=expected(r),public_interpreter=parsed,stderr=run.stderr));print(r['name'],parsed.get('result'),flush=True)
 record=dict(execution_root=str(ROOT.resolve()),stage='actual public pinned Interpreter Concatenate synthetic opcode evaluations; no private mirrors; Package/Device TermArg exposure only',upstream_revision=PIN,interpreter_binary=str(binary),interpreter_binary_sha256=sha(binary),generator_sha256=sha(Path(__file__)),fixtures_sha256=sha(HERE/'fixtures.py'),sources={str(p):sha(p)for p in[base/'src/main.rs',base/'Cargo.toml',base/'Cargo.lock']},upstream_sha256={str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))},rows=observations)
 (HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');verify()
if __name__=='__main__':main()
