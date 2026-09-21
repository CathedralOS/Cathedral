#!/usr/bin/env python3
"""Original public AML ObjectType/SizeOf observations, not private-body mirrors."""
import argparse,hashlib,importlib.util,json,os,subprocess,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cases(f):
 i=f.integer;n=f.name;op=f.op;ret=f.ret;pkg=f.pkg
 buf=lambda data,size=None:pkg(0x11,i(len(data)if size is None else size)+bytes(data))
 string=lambda text:b'\x0d'+text.encode()+b'\0'
 package=lambda *items:pkg(0x12,bytes([len(items)])+b''.join(items))
 rows=[]
 def add(label,prefix,body,expected):
  rows.append(dict(name=label,table_hex=(prefix+pkg(0x14,n('MAIN')+b'\0'+ret(body))).hex(),expected=expected))
 for label,value,typ,size in [('integer',i(42),1,None),('string',string('AB'),2,2),('buffer',buf([1,2]),3,2),('padded_buffer',buf([1,2],4),3,4),('package',package(i(1),i(2)),4,2),('empty_buffer',buf([]),3,0),('empty_string',string(''),2,0),('empty_package',package(),4,0)]:
  prefix=b'\x08'+n('OBJ')+value
  add('type_'+label,prefix,op(0x8e,n('OBJ')),typ)
  add('size_'+label,prefix,op(0x87,n('OBJ')),size)
  add('type_ref_'+label,prefix,op(0x8e,op(0x71,n('OBJ'))),typ)
  add('size_ref_'+label,prefix,op(0x87,op(0x71,n('OBJ'))),size)
 add('type_uninitialized_local',b'',op(0x8e,b'\x60'),0)
 for label,prefix,typ in [
  ('method',pkg(0x14,n('OBJ')+b'\0'+ret(i(1))),8),
  ('device',b'\x5b'+pkg(0x82,n('OBJ')),6),
  ('event',b'\x5b\x02'+n('OBJ'),7),
  ('processor',b'\x5b'+pkg(0x83,n('OBJ')+b'\0'*6),12),
  ('power',b'\x5b'+pkg(0x84,n('OBJ')+b'\0'*3),11),
  ('thermal',b'\x5b'+pkg(0x85,n('OBJ')),13),
  ('region',b'\x5b\x80'+n('OBJ')+b'\0'+i(0)+i(16),10),
  ('buffer_field',b'\x08'+n('BUF')+buf([1,2])+op(0x8c,n('BUF'),i(0),n('OBJ')),14),
 ]:add('type_'+label,prefix,op(0x8e,n('OBJ')),typ)
 prefix=b'\x08'+n('PKG')+package(n('OBJ'))+b'\x08'+n('OBJ')+buf([1,2,3])
 add('type_lexical_package',prefix,op(0x8e,op(0x88,n('PKG'),i(0),b'\0')),3)
 add('size_lexical_package',prefix,op(0x87,op(0x88,n('PKG'),i(0),b'\0')),3)
 add('size_oversized_initializer',b'\x08'+n('OBJ')+buf([1,2,3],1),op(0x87,n('OBJ')),3)
 return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');a=p.parse_args();repo=a.repository.resolve();base=repo/'tools/ports/acpi/aml-public-execution';fp=repo/'tools/ports/acpi/interpreter/execution/fixtures.py';s=importlib.util.spec_from_file_location('query_encoding',fp);f=importlib.util.module_from_spec(s);s.loader.exec_module(f)
 up=repo/'reference_code/rust-osdev/acpi';pin='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5';assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==pin
 assert not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=up,text=True).strip()
 target=Path('/tmp/cathedral-acpi-public-execution');subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(base/'Cargo.toml'),'--target-dir',str(target)],cwd=repo,check=True);binary=target/'release/cathedral-acpi-public-execution';observed={}
 with tempfile.TemporaryDirectory(prefix='cathedral-query-public-')as d:
  for row in cases(f):
   path=Path(d)/(row['name']+'.aml');path.write_bytes(bytes.fromhex(row['table_hex']));run=subprocess.run([str(binary),str(path),'2',''],env=dict(os.environ,CATHEDRAL_GENERIC_DESCRIBE='1'),capture_output=True,text=True,timeout=5);assert run.returncode==0,run.stderr
   values=dict(line.split('\t',1)for line in run.stdout.replace(str(repo)+'/','').splitlines());assert values['forbidden_calls']=='0' and values['created_mutexes']=='1'
   observed[row['name']]={**row,'observed':values,'agrees_on_value':row['expected']is not None and values.get('result')=='integer:'+str(row['expected'])};print(row['name'],json.dumps(values,sort_keys=True),flush=True)
 paths=[base/'src/main.rs',base/'Cargo.toml',base/'Cargo.lock',fp]
 record=dict(stage='actual public pinned AML ObjectType/SizeOf on host; no Omega result claimed',upstream_revision=pin,binary_sha256=sha(binary),source_sha256={str(p.relative_to(repo)):sha(p)for p in paths},generator_sha256=sha(Path(__file__)),upstream_sha256={str(p.relative_to(up)):sha(p)for p in sorted((up/'src').rglob('*.rs'))},rows=observed,agreements=sum(row['agrees_on_value']for row in observed.values()))
 path=HERE/'reference-verification.json'
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record,'reference drift'
 print(len(observed),'public query observations;',record['agreements'],'numeric agreements')
if __name__=='__main__':main()
