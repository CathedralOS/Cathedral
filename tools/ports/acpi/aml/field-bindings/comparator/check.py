#!/usr/bin/env python3
"""Execute the shared bridge's FieldBinding comparator with identity controls."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[5]
SHARED=ROOT/'tools/ports/acpi/interpreter/generic-execution/focused/bridge-atomicity/main.omg'
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def digest(text):return hashlib.sha256(text.encode()).hexdigest()
def fixture():
 original=SHARED.read_text().split('machine ba_binding',1)[0]
 source='\n'.join(line for line in original.splitlines()if not line.startswith('use ')or line.startswith('use aml::'))+'\n'
 # DeclarationKind is not otherwise needed by the shared comparator imports.
 source='use aml::field_model::DeclarationKind;\n'+source
 names=[]
 for kind,first,other in [('Region','region_object:17','region_object:18'),('Bank','region_object:17,selector_object:31','region_object:17,selector_object:32'),('Index','index_object:17,data_object:31','index_object:17,data_object:32')]:
  declaration='Declaration {kind:DeclarationKind::'+('Field'if kind=='Region'else kind)+',bank_value:42,source:Span {unit:7,start:10,end:20},field_list:Span {unit:7,start:15,end:20}}'
  field='Field {bit_offset:5,bit_length:8,source:Span {unit:7,start:15,end:20}}'
  for control in [False,True]:
   name='Suite::'+kind.lower()+('_control'if control else'_positive');names.append(name+'='+str(int(control)))
   source+=f'machine {name}(&mut self)->i32 {{let mut a:ObjectStore;let mut b:ObjectStore;a.space.object_count=1;b.space.object_count=1;'
   for var,payload in [('a',first),('b',other if control else first)]:
    source+=var+'.space.objects[0]=Object {value:Value::FieldUnit {binding:FieldBinding::'+kind+' {'+payload+'},declaration:'+declaration+',field:'+field+'}};'
   source+='let same:bool=fx_store(&a,&b);transition same {true -> (0) _ -> (1)}}\n'
 return source,names
def build():return 'machine build(builder:&mut Build){builder.package("field-binding-comparator");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});}\n'
def snapshot():return {str(p.relative_to(ROOT)):sha(p)for p in sorted([*(ROOT/'source/libraries/acpi/aml').glob('*.omg'),SHARED,HERE/'check.py'])}
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--verify',action='store_true');a=p.parse_args()
 binary=Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner');record_path=HERE/'checked-verification.json';source,names=fixture();before=snapshot();text=build();binary_hash=sha(binary)
 if not a.verify:
  started=time.monotonic()
  with tempfile.TemporaryDirectory(prefix='cathedral-field-binding-comparator-')as directory:
   work=Path(directory);(work/'main.omg').write_text(source);(work/'build.omg').write_text(text);command=[str(binary),str(work/'main.omg'),str(work/'build'),*names];lines=[]
   process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
   for line in process.stdout:print(line,end='',flush=True);lines.append(line)
   code=process.wait()
  record=dict(stage='checked interpreter',native_execution=False,execution_root=str(ROOT),source_sha256=before,source_unchanged=before==snapshot(),binary=str(binary),binary_sha256=binary_hash,fixture_sha256=digest(source),build_source=text,build_sha256=digest(text),command=command,selections=names,exit_code=code,output=''.join(lines),elapsed_seconds=time.monotonic()-started)
  record_path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 record=json.loads(record_path.read_text());original=text.replace(str(ROOT),record['execution_root'])
 assert record['source_unchanged']and record['source_sha256']==before and record['binary_sha256']==sha(record['binary'])
 assert record['fixture_sha256']==digest(source)and record['build_source']==original and record['build_sha256']==digest(original)
 assert record['exit_code']==0 and record['selections']==names,record['output']
 found=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',record['output'],re.M)
 assert len(found)==len(names)and [n+'='+e for n,e,o in found if e==o]==names
 print('PASS three shared bridge FieldBinding comparator pairs'+(' (retained record only)'if a.verify else''))
if __name__=='__main__':main()
