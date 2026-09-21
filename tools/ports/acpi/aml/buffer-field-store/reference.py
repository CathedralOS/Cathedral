#!/usr/bin/env python3
"""Actual pinned public Interpreter BufferField Store probes; all services trapped."""
import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import aml_encoding as aml
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
MAPPING=['source/libraries/acpi/aml/buffer_field_store.omg','source/libraries/acpi/aml/buffer_field_writes.omg','source/libraries/acpi/aml/byte_storage.omg']
def insert(backing,payload,start,count):
 mask=((1<<count)-1)<<start
 return ((int.from_bytes(backing,'little')&~mask)|((int.from_bytes(payload,'little')<<start)&mask)).to_bytes(len(backing),'little')
def cases():
 rows=[]
 def add(kind,bits,count,data,alias=False,start=None):
  start=(0 if count==2048 else 3)if start is None else start
  length=(start+count+7)//8;length=min(256,length+1)
  backing=bytes((165+i*37)%256 for i in range(length))
  source=str(data)if kind=='Integer'else data.hex()
  payload=(data&((1<<bits)-1)).to_bytes(bits//8,'little')if kind=='Integer'else data
  raw=data.to_bytes(8,'little')if kind=='Integer'else data
  name=f'{kind}_{bits}_{count}_{start}_{source[:24]}_{len(source)}_{alias}'
  strict=insert(backing,payload,start,count)
  pin=backing if kind=='String'else insert(backing,raw,start,count)
  classification='string-source-panic'if kind=='String'else'matches'if strict==pin else'raw-integer-width'
  rows.append(dict(name=name,kind=kind,bits=bits,count=count,start=start,source=source,backing_hex=backing.hex(),strict_hex=strict.hex(),pinned_hex=pin.hex(),alias=alias,classification=classification))
 for bits in [32,64]:
  for count in [1,9,32,33,64,65,127,2048]:
   for number in [0,1<<32,(1<<64)-1]:add('Integer',bits,count,number)
  for count in [1,9,65,2048]:
   for length in [0,1,8,9,256]:add('Buffer',bits,count,bytes((150+i*31)%256 for i in range(length)))
  for count in [9,65,2048]:
   for length in [0,1,255,256]:add('String',bits,count,b'A'*length)
 for start in [0,3,7,8]:add('Buffer',64,65,bytes.fromhex('123456789abcdef011'),True,start)
 assert len({r['name']for r in rows})==len(rows)
 return rows
def table(row):
 data=aml.named('BACK',{'buffer':row['backing_hex']})
 if row['alias']:data+=aml.named('SRCE',{'buffer':row['source']})+b'\x06'+aml.name('SRCE')+aml.name('ALIA')
 body=b'\x5b\x13'+aml.name('BACK')+aml.integer(row['start'])+aml.integer(row['count'])+aml.name('FLD_')
 body+=b'\x70'+(aml.name('ALIA')if row['alias']else b'\x68')+aml.name('FLD_')+b'\xa4\x00'
 return data+aml.pkg(0x14,aml.name('MAIN')+b'\x01'+body)
def validate(row,observed):
 assert observed['load']=='ok'and observed['forbidden_calls']=='0'and observed['created_mutexes']=='1',observed
 assert observed['before']=='buffer:'+row['backing_hex'] and observed['after']=='buffer:'+row['pinned_hex'],observed
 assert observed['same_backing']=='true' and observed['argument_before']==observed['argument_after'],observed
 expected_arg=row['kind'].lower()+':'+row['source'];assert observed['argument_before']==expected_arg,observed
 assert observed['result']==('panic'if row['kind']=='String'else'integer:0'),observed
 if row['alias']:assert observed['alias_identity']==observed['source_distinct_backing']=='true'and observed['named_source_after']=='buffer:'+row['source'],observed
def upstream(repository):
 up=repository.resolve()/'reference_code/rust-osdev/acpi';assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=up,text=True).strip()==PIN
 source={str(p.relative_to(up)):sha(p)for p in sorted((up/'src').rglob('*.rs'))}
 for n in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:source[n]=sha(up/n)
 for n,h in source.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+n],cwd=up)).hexdigest()==h
 return up,source
def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--write',action='store_true');p.add_argument('--smoke',action='store_true');a=p.parse_args();up,source=upstream(a.repository);mapping={n:sha(a.repository/n)for n in MAPPING};inputs={n:sha(HERE/n)for n in ['reference.py','reference.rs','aml_encoding.py']}
 target=Path('/tmp/cathedral-buffer-field-store-public')
 with tempfile.TemporaryDirectory(prefix='buffer-field-store-public-')as d:
  work=Path(d);manifest=f'[package]\nname="cathedral-buffer-field-store-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n';(work/'Cargo.toml').write_text(manifest);lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  inputs['reference.Cargo.lock']=sha(lock)
  subprocess.run(['cargo','+nightly-2026-09-04','build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
  binary=target/'release/reference';rows=[]
  selected=cases()
  if a.smoke:selected=[selected[i]for i in [0,11,48,57,89,112]]
  for row in selected:
   data=table(row);file=work/'table.aml';file.write_bytes(data)
   r=subprocess.run([str(binary),str(file),'1'if row['bits']==32 else'2',row['kind'],row['source'],'alias'if row['alias']else'argument'],capture_output=True,text=True,timeout=5,check=True);observed=dict(line.split('\t',1)for line in r.stdout.splitlines());validate(row,observed)
   rows.append(dict(**row,aml_hex=data.hex(),observed=observed))
 assert mapping=={n:sha(a.repository/n)for n in MAPPING} and inputs=={n:sha(HERE/n)for n in inputs}
 record=dict(execution_root=str(a.repository.resolve()),probe_root=str(HERE),build_text=manifest,build_sha256=hashlib.sha256(manifest.encode()).hexdigest(),production_mapping_sha256=mapping,notes=['Raw public Integer arguments are not normalized before entry; revision1 exposes pinned unconditional eight-byte writes.','Source/backing alias is not invoked: pinned gain_mut paths could create overlapping mutable references. Omega tests that case through detached snapshots; named aliases here refer to a distinct source Buffer.','Each row gets a fresh interpreter; String panic is caught and backing observed unchanged after unwinding. No String backing or invalid UTF8 mutation.'],stage='actual public Interpreter::new/load_table/evaluate, synthetic AML, all hardware and services trapped; no private mirrors',pin=PIN,upstream_sha256=source,binary_sha256=sha(binary),source_sha256={p.name:sha(p)for p in [HERE/'reference.py',HERE/'reference.rs',HERE/'aml_encoding.py',lock]},rows=rows)
 path=HERE/('reference-smoke.json'if a.smoke else'reference-verification.json')
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record
 print('PASS',len(rows),'public Store observations;', {k:sum(r['classification']==k for r in rows)for k in ['matches','raw-integer-width','string-source-panic']})
if __name__=='__main__':main()
