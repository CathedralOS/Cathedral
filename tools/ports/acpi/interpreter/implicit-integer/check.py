#!/usr/bin/env python3
"""Primary-rule vectors and actual checked Omega bodies, including negative controls."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from collections import Counter
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
RUNNER=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner')
COMPILER=Path('/tmp/cathedral-omega-eaa7993/release/omega')
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def expected(data,length,bits):
 if length>256:return ('Capacity',0)
 if length==0:return ('Empty',0)
 if any(b==0 or b>=128 for b in data[:length]):return ('Encoding',0)
 prefix=[]
 for b in data[:min(length,bits//4)]:
  if chr(b) not in '0123456789abcdefABCDEF':break
  prefix.append(chr(b))
 return ('Integer',int(''.join(prefix),16) if prefix else 0)
def cases():
 rows=[]
 def add(name,data,bits=64,length=None):
  if isinstance(data,str):data=list(data.encode('ascii'))
  length=len(data) if length is None else length
  kind,value=expected(data,length,bits)
  rows.append(dict(name=name,bits=bits,data=data,length=length,kind=kind,value=value))
 for bits in [32,64]:
  for b in range(256):add(f'first_{bits}_{b}',[b,49,65],bits)
  for i,s in enumerate(['','0','1','10','1234','aBcDeF','FFFFFFFF','FFFFFFFFFFFFFFFF','FFFFFFFFFFFFFFFFF','123456789ABCDEF012345','00000000000000001','0x12','0XFF','+123','-1',' 12','\t12','A G','G123','12.3','12gFF','12\nF']):add(f'text_{bits}_{i}',s,bits)
  for count in [7,8,9,15,16,17,255,256]:add(f'ones_{bits}_{count}','F'*count,bits)
  for position in [0,1,7,8,15,16,254,255]:
   for byte in [0,128,255]:
    data=[65]*256;data[position]=byte;add(f'invalid_{bits}_{position}_{byte}',data,bits)
  for length in [257,1<<63,(1<<64)-1]:add(f'extent_{bits}_{length}',[49]*256,bits,length)
  add(f'unused_{bits}',[49,50]+[255]*254,bits,2)
  add(f'delimiter_then_invalid_{bits}',[49,71,255],bits)
 return rows
IMPORTS='use interpreter::implicit_integer;\nuse interpreter::implicit_integer::Conversion;\nuse interpreter::implicit_integer::Error;\nuse interpreter::integers::IntegerSize;\n'
HELPERS='''
machine fill(input:&mut [u8;256],index:u64,count:u64,byte:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {
 _=fill_at(input,index,count,byte);
 transition index<count {true -> fill(input,index+1,count,byte) _ -> (0)}
}
machine fill_at(input:&mut [u8;256],index:u64,count:u64,byte:u8)->u8 {
 transition index<count && index<256 {true -> put(input,index,byte) _ -> (0)}
 state put(input:&mut [u8;256],index:u64,byte:u8)->u8 {input[index]=byte;0}
}
machine matches(value:Conversion,kind:u8,number:u64)->bool {
 transition value {
  Conversion::Integer {value as actual} -> (kind==0 && actual==number)
  Conversion::Failure {reason} -> error(reason,kind)
 }
 state error(reason:Error,kind:u8)->bool {
  transition reason {Error::Capacity -> (kind==1) Error::Empty -> (kind==2) Error::Encoding -> (kind==3)}
 }
}
'''
def body(row,control):
 kind={'Integer':0,'Capacity':1,'Empty':2,'Encoding':3}[row['kind']];number=row['value']
 if control:
  if kind==0:number^=1
  else:kind=0
 data=row['data']+[0]*(256-len(row['data']));base=Counter(data).most_common(1)[0][0]
 assignments=('_=fill(&mut input,0,256,'+str(base)+');')if base else''
 assignments+=''.join(f'input[{i}]={v};'for i,v in enumerate(data)if v!=base)
 size='FourBytes'if row['bits']==32 else'EightBytes'
 return 'let mut input:[u8;256];'+assignments+f'let value:Conversion=implicit_integer::from_string(IntegerSize::{size},&input,{row["length"]});let good:bool=matches(value,{kind},{number});transition good {{true -> (0) _ -> (1)}}'
def authored(rows):
 text=IMPORTS+HELPERS+'data Suite{}\n';names=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');names.append(name+'='+str(int(control)));text+='machine '+name+'(&mut self)->i32{'+body(row,control)+'}\n'
 return text,names
def build():return 'machine build(builder:&mut Build){builder.application("cathedral-implicit-integer-checks");builder.freestanding=true;builder.depend_as("interpreter",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/interpreter')+'"});}'
def snapshot():
 paths=[HERE/'check.py',HERE/'cases.json']+[ROOT/'source/libraries/acpi/interpreter'/n for n in ['build.omg','integers.omg','implicit_integer.omg']]+[ROOT/'tools/ports/acpi/interpreter/execution'/n for n in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(p.relative_to(ROOT)):sha(p)for p in paths}
def validate(out,names):
 assert out.count('CHECKED authored package and dependency bodies;')==1
 actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',out,re.M);assert len(actual)==len(names)
 for (name,want,got),selection in zip(actual,names):assert name+'='+want==selection and want==got
def selected(rows):return [next(r for r in rows if r['name']==n)for n in ['text_64_8','text_32_9','invalid_64_255_255']]
def const_source(row,control):return IMPORTS+HELPERS+'machine ii_constant_result()->i32{'+body(row,control)+'}\nconst RESULT:i32=ii_constant_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
def run():
 rows=cases();assert json.loads((HERE/'cases.json').read_text())==rows;inputs=snapshot();runner=sha(RUNNER);compiler=sha(COMPILER);source,names=authored(rows);proofs=[];batches=[];start=time.monotonic()
 with tempfile.TemporaryDirectory(prefix='cathedral-implicit-integer-')as d:
  work=Path(d);(work/'build.omg').write_text(build())
  for start_at in range(0,len(rows),30):
   batch=rows[start_at:start_at+30];text,selections=authored(batch);(work/'main.omg').write_text(text)
   r=subprocess.run([str(RUNNER),str(work/'main.omg'),str(work/'build'),*selections],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));out=r.stdout+r.stderr;assert r.returncode==0,out;validate(out,selections)
   batches.append(dict(cases=[r['name']for r in batch],source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=out));print('PASS',start_at+len(batch),'/',len(rows),'checked pairs',flush=True)
  for row in selected(rows):
   for control in [False,True]:
    text=const_source(row,control);(work/'main.omg').write_text(text);r=subprocess.run([str(COMPILER),'--check',str(work/'main.omg')],capture_output=True,text=True);output=r.stdout+r.stderr
    if control:assert r.returncode!=0 and 'cannot prove requires contract'in output and '1 == 0'in output,output
    else:assert r.returncode==0,output
    proofs.append(dict(case=row['name'],control=control,source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=output));print('PASS const',row['name'],control,flush=True)
 assert inputs==snapshot()and runner==sha(RUNNER)and compiler==sha(COMPILER)
 (HERE/'verification.json').write_text(json.dumps(dict(omega_revision=PIN,input_sha256=inputs,runner_sha256=runner,compiler_sha256=compiler,source_sha256=hashlib.sha256(source.encode()).hexdigest(),positive_count=len(rows),control_count=len(rows),batches=batches,constant_proofs=proofs,seconds=round(time.monotonic()-start,3)),indent=2,sort_keys=True)+'\n')
def verify():
 rows=cases();assert json.loads((HERE/'cases.json').read_text())==rows;r=json.loads((HERE/'verification.json').read_text());source,names=authored(rows)
 assert r['omega_revision']==PIN and r['input_sha256']==snapshot()and r['runner_sha256']==sha(RUNNER)and r['compiler_sha256']==sha(COMPILER)
 assert r['positive_count']==r['control_count']==len(rows)and r['source_sha256']==hashlib.sha256(source.encode()).hexdigest();assert len(r['constant_proofs'])==6
 seen=[];lookup={row['name']:row for row in rows}
 for batch in r['batches']:
  text,selections=authored([lookup[n]for n in batch['cases']]);assert batch['source_sha256']==hashlib.sha256(text.encode()).hexdigest();validate(batch['output'],selections);seen+=batch['cases']
 assert seen==[row['name']for row in rows]
 for row,pair in zip(selected(rows),zip(r['constant_proofs'][::2],r['constant_proofs'][1::2])):
  for control,proof in zip([False,True],pair):
   assert proof['case']==row['name']and proof['control']==control and proof['source_sha256']==hashlib.sha256(const_source(row,control).encode()).hexdigest()
   assert ('cannot prove requires contract'in proof['output']and'1 == 0'in proof['output'])if control else'compiled 'in proof['output']
 print('PASS',len(rows),'checked pairs and 3 const pairs; exact current hashes')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--write-cases',action='store_true');p.add_argument('--verify-record',action='store_true');a=p.parse_args()
 if a.write_cases:(HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n')
 elif a.verify_record:verify()
 else:run()
