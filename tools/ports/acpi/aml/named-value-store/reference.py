#!/usr/bin/env python3
"""Actual public pinned Interpreter named Store; independent primary-policy oracle."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import aml_encoding as aml
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
MAPPING=['source/libraries/acpi/aml/named_value_store.omg','source/libraries/acpi/aml/implicit_conversions.omg','source/libraries/acpi/aml/buffer_target_values.omg','source/libraries/acpi/aml/byte_storage.omg']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def describe(kind,data):return kind.lower()+':'+(str(data)if kind=='Integer'else data.hex())
def primary(kind,extent,source,data,bits):
 mask=(1<<bits)-1
 if kind=='Integer':
  if source=='Integer':return describe(kind,data&mask),None
  if not data:return None,'Empty'
  if source=='Buffer':return describe(kind,int.from_bytes(data[:bits//8],'little')),None
  digits=''
  for b in data[:bits//4]:
   if chr(b)not in '0123456789abcdefABCDEF':break
   digits+=chr(b)
  return describe(kind,int(digits or '0',16)),None
 if kind=='String':
  if source=='Integer':out=f'{data&mask:0{bits//4}X}'.encode()
  elif source=='Buffer':out=b' '.join(f'{v:02X}'.encode()for v in data)
  else:out=data
  return (None,'Capacity')if len(out)>256 else(describe(kind,out),None)
 if not extent:return None,'excluded-zero-extent'
 if source=='Buffer':return None,'excluded-buffer-source'
 if source=='String'and not data:return None,'excluded-empty-string'
 raw=(data&mask).to_bytes(bits//8,'little')if source=='Integer'else data
 return describe(kind,raw[:extent]+bytes(max(0,extent-len(raw)))),None

def cases():
 rows=[]
 targets=[('Integer',99)]+[('String',b'Q'*n)for n in [0,3,256]]+[('Buffer',bytes([165])*n)for n in [0,1,4,8,256]]
 sources=[('Integer',v)for v in [0,0x4847464544434241,(1<<64)-1]]+[('String',v)for v in [b'',b'A',b'1A',b'0x10',b'FFFFFFFFFFFFFFFFZ',b'G'+b'A'*255]]+[('Buffer',v)for v in [b'',b'\0',b'\x7f\x80\xff',bytes(range(9)),bytes([255])*85,bytes([1])*86,bytes(range(256))]]
 def add(kind,old,source,data,bits,alias=False):
  strict,error=primary(kind,len(old)if kind!='Integer'else 0,source,data,bits)
  raw=data.to_bytes(8,'little')if source=='Integer'else data
  if kind=='Integer':expected=describe(kind,int.from_bytes(raw[:8],'little'))
  elif kind=='String':expected=describe(kind,raw.decode('utf8',errors='replace').split('\0')[0].encode())
  else:expected=describe(kind,raw)
  classification='excluded-profile-observation'if error and error.startswith('excluded-')else'primary-rejection-pin-success'if error else'matches'if strict==expected else'conversion-policy-difference'
  rows.append(dict(name=f'case_{len(rows):03d}',target_kind=kind,target=str(old)if kind=='Integer'else old.hex(),kind=source,source=str(data)if source=='Integer'else data.hex(),bits=bits,alias=alias,strict=strict,strict_error=error,pinned=expected,classification=classification))
 for bits in [32,64]:
  for kind,old in targets:
   for source,data in sources:add(kind,old,source,data,bits)
 for kind,old in [('Integer',99),('String',b'old'),('Buffer',bytes([165])*8)]:
  for source,data in [('Integer',0x41424344),('String',b'1A'),('Buffer',b'\x01\x02')]:add(kind,old,source,data,64,True)
 return rows

def value(kind,data):return int(data)if kind=='Integer'else{'string_hex'if kind=='String'else'buffer':data}
def table(row):
 data=aml.named('TARG',value(row['target_kind'],row['target']))
 if row['alias']:data+=aml.named('SRCE',value(row['kind'],row['source']))+b'\x06'+aml.name('SRCE')+aml.name('ALIA')
 body=b'\x70'+(aml.name('ALIA')if row['alias']else b'\x68')+aml.name('TARG')+b'\xa4\x68'
 return data+aml.pkg(0x14,aml.name('MAIN')+b'\x01'+body)
def validate(row,observed):
 assert observed['load']=='ok'and observed['forbidden_calls']=='0'and observed['created_mutexes']=='1',observed
 assert observed['before']==row['target_kind'].lower()+':'+row['target']and observed['after']==row['pinned'],observed
 assert observed['same_target']=='true'and observed['argument_before']==observed['argument_after'],observed
 expected=row['kind'].lower()+':'+row['source'];assert observed['argument_before']==expected and observed['result']==expected,observed
 if row['alias']:assert observed['alias_identity']==observed['source_distinct_target']=='true'and observed['named_source_after']==expected,observed

def upstream(repository):
 up=repository.resolve()/'reference_code/rust-osdev/acpi';assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 source={str(p.relative_to(up)):sha(p)for p in sorted((up/'src').rglob('*.rs'))}
 for n in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:source[n]=sha(up/n)
 for n,h in source.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+n],cwd=up)).hexdigest()==h
 return up,source

def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');p.add_argument('--smoke',action='store_true');a=p.parse_args();up,source=upstream(a.repository);mapping={n:sha(a.repository/n)for n in MAPPING};inputs={n:sha(HERE/n)for n in ['reference.py','reference.rs','aml_encoding.py']}
 target=Path('/tmp/cathedral-named-value-store-public')
 with tempfile.TemporaryDirectory(prefix='named-value-store-public-')as d:
  work=Path(d);manifest=f'[package]\nname="cathedral-named-value-store-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n';(work/'Cargo.toml').write_text(manifest);lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  inputs['reference.Cargo.lock']=sha(lock)
  subprocess.run(['cargo','+nightly-2026-09-04','build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
  binary=target/'release/reference';rows=[];selected=cases()
  if a.smoke:selected=[selected[i]for i in [0,4,9,16,19,65,81,144,288,296]]
  for row in selected:
   data=table(row);file=work/'table.aml';file.write_bytes(data)
   r=subprocess.run([str(binary),str(file),'1'if row['bits']==32 else'2',row['kind'],row['source'],'alias'if row['alias']else'argument'],capture_output=True,text=True,timeout=5,check=True);observed=dict(line.split('\t',1)for line in r.stdout.splitlines());validate(row,observed);rows.append(dict(**row,aml_hex=data.hex(),observed=observed))
 assert mapping=={n:sha(a.repository/n)for n in MAPPING}and inputs=={n:sha(HERE/n)for n in inputs}
 record=dict(execution_root=str(a.repository.resolve()),probe_root=str(HERE),build_text=manifest,build_sha256=hashlib.sha256(manifest.encode()).hexdigest(),production_mapping_sha256=mapping,notes=['Actual Store(Arg0,TARG); Return(Arg0), with named aliases of distinct sources in nine rows. This observes target mutation and unchanged source; it does not claim the Store expression return value.','Raw public Integer arguments retain all64bits at entry; both AML revisions are observed.','Self-store is not invoked because pinned unsafe mutable target access may overlap source immutable access. Detached Omega self-tests cover that separately.','Primary rejection and excluded-profile observations are distinguished. The pin performs raw byte conversion, UTF8-lossy String truncation at NUL, and Buffer replacement.'],stage='actual public Interpreter::new/load_table/evaluate, synthetic AML, all services trapped; no private mirrors',pin=PIN,upstream_sha256=source,binary_sha256=sha(binary),source_sha256={n:sha(HERE/n)for n in inputs},rows=rows)
 path=HERE/('reference-smoke.json'if a.smoke else'reference-verification.json')
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record
 print('PASS',len(rows),'public named Store observations;', {k:sum(r['classification']==k for r in rows)for k in sorted({r['classification']for r in rows})})
if __name__=='__main__':main()
