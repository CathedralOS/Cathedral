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
def cases():
 rows=[]
 def integer(bits,number):
  expected=f'{number&((1<<bits)-1):0{bits//4}X}'
  rows.append(dict(name=f'integer_{bits}_{number}',kind='integer',bits=bits,number=number,data=[],length=0,expected=list(expected.encode())))
 for bits in [32,64]:
  for number in sorted(set([0,1,9,10,15,16,255,256,(1<<31),(1<<32),(1<<63),(1<<32)-1,(1<<64)-1,0x123456789ABCDEF0]+[int(str(hex(n))[2:]*16,16)for n in range(16)])):
   integer(bits,number)
 def buffer(name,data,length=None):
  length=len(data)if length is None else length
  expected=None if length>85 else list(' '.join(f'{b:02X}'for b in data[:length]).encode())
  rows.append(dict(name=name,kind='buffer',data=data,length=length,expected=expected))
 for byte in range(256):buffer(f'byte_{byte}',[byte])
 for count in [0,2,3,4,8,16,32,64,84,85,86,255,256]:
  buffer(f'pattern_{count}',[(i*37+19)%256 for i in range(count)])
 for count in [2,85]:
  for byte in [0,255]:buffer(f'uniform_{count}_{byte}',[byte]*count)
 for length in [257,1<<63,(1<<64)-1]:buffer(f'invalid_{length}',[1]*256,length)
 buffer('unused_poison',[0,255]+[221]*254,2)
 buffer('empty_poison',[255]*256,0)
 return rows
IMPORTS='use interpreter::implicit_strings;\nuse interpreter::implicit_strings::StringResult;\nuse interpreter::integers::IntegerSize;\n'
HELPERS="""
machine same_bytes(actual:&[u8;256],expected:&[u8;256],index:u64,count:u64,same:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {
 let next:bool=same_at(actual,expected,index,count,same);
 transition index<count {true -> same_bytes(actual,expected,index+1,count,next) _ -> (same)}
}
machine same_at(actual:&[u8;256],expected:&[u8;256],index:u64,count:u64,same:bool)->bool {
 transition index<count && index<256 {true -> (same && actual[index]==expected[index]) _ -> (same)}
}
machine matches(value:StringResult,capacity:bool,length:u64,expected:&[u8;256])->bool {
 transition value {
  StringResult::Capacity -> (capacity)
  StringResult::String {length as actual_length,bytes} -> check(&bytes,actual_length,capacity,length,expected)
 }
 state check(actual:&[u8;256],actual_length:u64,capacity:bool,length:u64,expected:&[u8;256])->bool {
  let same:bool=same_bytes(actual,expected,0,256,true);!capacity && actual_length==length && same
 }
}
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
"""
def assignments(name,data):
 data=data+[0]*(256-len(data));base=Counter(data).most_common(1)[0][0]
 code=('let mut '+name+':[u8;256];')+('_=fill(&mut '+name+',0,256,'+str(base)+');'if base else'')
 return code+''.join(f'{name}[{i}]={v};'for i,v in enumerate(data)if v!=base)
def body(row,control):
 expected=row['expected'];capacity=expected is None;length=0 if capacity else len(expected)
 out=[]if capacity else expected.copy()
 if control:
  if capacity:capacity=False
  else:out += [0]*(256-len(out));out[255]=1
 code=assignments('input',row['data'])+assignments('expected',out)
 if row['kind']=='integer':
  size='FourBytes'if row['bits']==32 else'EightBytes';call=f'implicit_strings::from_integer(IntegerSize::{size},{row["number"]})'
 else:call=f'implicit_strings::from_buffer(&input,{row["length"]})'
 return code+f'let value:StringResult={call};let good:bool=matches(value,{str(capacity).lower()},{length},&expected);transition good {{true -> (0) _ -> (1)}}'
def authored(rows):
 text=IMPORTS+HELPERS+'data Suite{}\n';names=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');names.append(name+'='+str(int(control)));text+='machine '+name+'(&mut self)->i32{'+body(row,control)+'}\n'
 return text,names
def build():return 'machine build(builder:&mut Build){builder.application("cathedral-implicit-strings-checks");builder.freestanding=true;builder.depend_as("interpreter",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/interpreter')+'"});}'
def snapshot():
 paths=[HERE/'check.py',HERE/'cases.json']+[ROOT/'source/libraries/acpi/interpreter'/n for n in ['build.omg','integers.omg','implicit_strings.omg']]+[ROOT/'tools/ports/acpi/interpreter/execution'/n for n in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(p.relative_to(ROOT)):sha(p)for p in paths}
def validate(out,names):
 assert out.count('CHECKED authored package and dependency bodies;')==1
 actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',out,re.M);assert len(actual)==len(names)
 for (name,want,got),selection in zip(actual,names):assert name+'='+want==selection and want==got
def selected(rows):return [next(r for r in rows if r['name']==n)for n in ['integer_32_18446744073709551615','pattern_85','invalid_18446744073709551615']]
def const_source(row,control):return IMPORTS+HELPERS+'machine is_constant_result()->i32{'+body(row,control)+'}\nconst RESULT:i32=is_constant_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
def run():
 rows=cases();assert json.loads((HERE/'cases.json').read_text())==rows;inputs=snapshot();runner=sha(RUNNER);compiler=sha(COMPILER);source,names=authored(rows);proofs=[];batches=[];start=time.monotonic()
 with tempfile.TemporaryDirectory(prefix='cathedral-implicit-strings-')as d:
  work=Path(d);(work/'build.omg').write_text(build())
  for start_at in range(0,len(rows),10):
   batch=rows[start_at:start_at+10];text,selections=authored(batch);(work/'main.omg').write_text(text)
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
