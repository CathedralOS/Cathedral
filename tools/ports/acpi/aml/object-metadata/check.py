#!/usr/bin/env python3
"""Actual public Rust observations and source-bound Omega body/control checks."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5';OMEGA='eaa7993a23623cd8fabf45350340479c5c9c7879'
UP=ROOT/'reference_code/rust-osdev/acpi';RUNNER=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner');COMPILER=Path('/tmp/cathedral-omega-eaa7993/release/omega')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def public(write=False):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 upstream={str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))}
 for name in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:upstream[name]=sha(UP/name)
 for name,digest in upstream.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+name],cwd=UP)).hexdigest()==digest
 with tempfile.TemporaryDirectory(prefix='cathedral-metadata-rust-')as d:
  work=Path(d);(work/'Cargo.toml').write_text('[package]\nname="cathedral-object-metadata-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={path="'+str(UP)+'"}\n[[bin]]\nname="reference"\npath="'+str(HERE/'reference.rs')+'"\n');(work/'Cargo.lock').write_bytes((HERE/'reference.Cargo.lock').read_bytes())
  subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-object-metadata-reference'],check=True)
 binary=Path('/tmp/cathedral-object-metadata-reference/release/reference');output=subprocess.check_output([str(binary)],text=True);rows=[]
 for line in output.splitlines():
  kind,raw,*fields=line.split('\t');rows.append(dict(kind=kind,raw=int(raw),fields=fields))
 assert [r['raw']for r in rows[:256]]==list(range(256));assert len(rows)==448
 assert [r['raw']for r in rows[256:]]==[prefix|low for prefix in [0,32,1<<31,1<<32,1<<63,((1<<64)-1)&~31]for low in range(32)]
 value=dict(pin=PIN,upstream_sha256=upstream,source_sha256={p.name:sha(p)for p in [HERE/'check.py',HERE/'reference.rs',HERE/'reference.Cargo.lock']},binary_sha256=sha(binary),rows=rows)
 path=HERE/'public-verification.json'
 if write:path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==value
 print('PASS 448 actual public Rust observations',flush=True)
 return rows
def snapshot():
 paths=list(HERE.glob('*.py'))+[HERE/n for n in ['reference.rs','reference.Cargo.lock','public-verification.json']]+[ROOT/'source/libraries/acpi/aml'/n for n in ['build.omg','object_metadata.omg']]+[ROOT/'source/libraries/acpi/interpreter/build.omg']+[ROOT/'tools/ports/acpi/interpreter/execution'/n for n in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(p.relative_to(ROOT)):sha(p)for p in sorted(paths)}
HELPERS="""
machine check_method(raw:u8,args:u8,serialized:bool,level:u8)->bool {
 let v:object_metadata::MethodFacts=object_metadata::method_flags(raw);
 v.argument_count==args && v.serialized==serialized && v.sync_level==level
}
machine check_status(raw:u64,present:bool,enabled:bool,ui:bool,functioning:bool,battery:bool)->bool {
 let v:object_metadata::StatusFacts=object_metadata::device_status(raw);
 v.present==present && v.enabled==enabled && v.show_in_ui==ui && v.functioning==functioning && v.battery_present==battery
}
"""
def body(rows,control):
 calls=[]
 for i,row in enumerate(rows):
  method=row['kind']=='method';values=row['fields'].copy()
  if control and i==len(rows)-1:values[1]='false'if values[1]=='true'else'true'
  calls.append('check_'+('method'if method else'status')+'('+str(row['raw'])+','+','.join(values)+')')
 return 'let good:bool='+' && '.join(calls)+';\ntransition good {true -> (0) _ -> (1)}\n'
def build():return 'machine build(builder:&mut Build){builder.application("cathedral-object-metadata-checks");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});}'
def authored(rows):
 text='use aml::object_metadata;\n'+HELPERS+'data Suite{}\n';names=[]
 for start in range(0,len(rows),8):
  for control in [False,True]:
   name=f'Suite::group_{start//8}_{"control"if control else"positive"}';names.append(name+'='+str(int(control)));text+='machine '+name+'(&mut self)->i32{\n'+body(rows[start:start+8],control)+'}\n'
 return text,names
def const_source(rows,control):return 'use aml::object_metadata;\n'+HELPERS+'machine test_result()->i32{\n'+body([rows[255],rows[-1]],control)+'}\nconst RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
def validate(output,names):
 assert 'CHECKED authored package and dependency bodies;'in output
 actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',output,re.M)
 assert len(actual)==len(names)
 for (name,expected,observed),wanted in zip(actual,names):assert name+'='+expected==wanted and expected==observed

def run(rows):
 inputs=snapshot();binary=sha(RUNNER);compiler=sha(COMPILER);text,names=authored(rows);proofs=[];start=time.monotonic()
 with tempfile.TemporaryDirectory(prefix='cathedral-metadata-omega-')as d:
  work=Path(d);(work/'build.omg').write_text(build());(work/'main.omg').write_text(text)
  result=subprocess.run([str(RUNNER),str(work/'main.omg'),str(work/'build'),*names],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));output=result.stdout+result.stderr;assert result.returncode==0,output;validate(output,names);print(output,flush=True)
  for control in [False,True]:
   source=const_source(rows,control);(work/'main.omg').write_text(source);r=subprocess.run([str(COMPILER),'--check',str(work/'main.omg')],capture_output=True,text=True);out=r.stdout+r.stderr
   if control:assert r.returncode!=0 and 'cannot prove requires contract'in out and '1 == 0'in out,out
   else:assert r.returncode==0,out
   proofs.append(dict(control=control,source_sha256=hashlib.sha256(source.encode()).hexdigest(),output=out))
 assert inputs==snapshot() and binary==sha(RUNNER) and compiler==sha(COMPILER)
 record=dict(omega_revision=OMEGA,input_sha256=inputs,runner_sha256=binary,compiler_sha256=compiler,source_sha256=hashlib.sha256(text.encode()).hexdigest(),positive_count=56,control_count=56,scenario_count=448,output=output,constant_proofs=proofs,seconds=round(time.monotonic()-start,3))
 (HERE/'verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');print('PASS 448 scenarios in56pairs and1constant pair',flush=True)
def verify():
 r=json.loads((HERE/'verification.json').read_text());rows=json.loads((HERE/'public-verification.json').read_text())['rows'];text,names=authored(rows)
 assert r['omega_revision']==OMEGA and r['input_sha256']==snapshot();assert r['runner_sha256']==sha(RUNNER)and r['compiler_sha256']==sha(COMPILER)
 assert r['positive_count']==r['control_count']==56 and r['scenario_count']==len(rows)==448
 assert r['source_sha256']==hashlib.sha256(text.encode()).hexdigest();validate(r['output'],names)
 assert len(r['constant_proofs'])==2
 for control,proof in zip([False,True],r['constant_proofs']):
  assert proof['control']==control and proof['source_sha256']==hashlib.sha256(const_source(rows,control).encode()).hexdigest()
  assert ('cannot prove requires contract'in proof['output']and'1 == 0'in proof['output'])if control else'compiled 'in proof['output']
 print('PASS current record hashes;448 scenarios,56pairs,1const pair')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--write-public',action='store_true');p.add_argument('--public-only',action='store_true');p.add_argument('--verify-record',action='store_true');a=p.parse_args()
 if a.verify_record:verify()
 else:
  rows=public(a.write_public)
  if not a.public_only:run(rows)
