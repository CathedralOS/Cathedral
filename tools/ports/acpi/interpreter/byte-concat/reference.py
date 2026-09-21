#!/usr/bin/env python3
"""Real public Object conversion calls plus labelled private append-expression mirrors."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;UP=ROOT/'reference_code/rust-osdev/acpi'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==fixtures.PIN
hashes={}
for name in ['src/aml/mod.rs','src/aml/object.rs','Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:
 data=subprocess.check_output(['git','show',fixtures.PIN+':'+name],cwd=UP);assert data==(UP/name).read_bytes();hashes[name]=hashlib.sha256(data).hexdigest()
original=(UP/'src/aml/mod.rs').read_text();private=original[original.index('    fn do_concat('):original.index('    fn do_from_bcd(')]
for expression in ['buffer.extend_from_slice(&source1.to_le_bytes());','buffer.extend_from_slice(&(source1 as u32).to_le_bytes());','buffer.extend(source2.to_buffer(self.integer_size)?);','Object::String(source1 + &source2).wrap()']:assert expression in private
preamble='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Private do_concat expression mirrors, with real public Object conversions.
// Not a call to do_concat, an AML opcode, a MethodContext or Store.
use acpi::aml::{IntegerSize,object::Object};
fn mirror_integer(source1:&Object,source2:&Object,integer_size:IntegerSize)->Vec<u8>{
 let source1=source1.as_integer().unwrap();let source2=source2.to_integer(integer_size).unwrap();
 let mut buffer=Vec::new();
 if integer_size==IntegerSize::EightBytes {buffer.extend_from_slice(&source1.to_le_bytes());buffer.extend_from_slice(&source2.to_le_bytes());}
 else {buffer.extend_from_slice(&(source1 as u32).to_le_bytes());buffer.extend_from_slice(&(source2 as u32).to_le_bytes());}
 buffer
}
fn mirror_buffer(source1:&Object,source2:&Object,integer_size:IntegerSize)->Vec<u8>{
 let mut buffer=source1.as_buffer().unwrap().to_vec();buffer.extend(source2.to_buffer(integer_size).unwrap());buffer
}
fn mirror_string(source1:&Object,source2:&Object)->Vec<u8>{
 let source1=match source1{Object::String(value)=>value.clone(),_=>panic!("fixture type")};
 let source2=match source2{Object::String(value)=>value.clone(),_=>panic!("fixture type")};
 (source1+&source2).into_bytes()
}
fn main(){
'''
lines=[preamble];expected={};omitted=[];calls=0
for row in fixtures.cases():
 name=row['name'];size='IntegerSize::FourBytes'if row['width']==4 else'IntegerSize::EightBytes'
 if row['kind']=='integer':
  objects=[f'Object::Integer({row[key]}u64)'for key in ['left','right']];width=row['width'];mask=(1<<(width*8))-1;want=list((row['left']&mask).to_bytes(width,'little')+(row['right']&mask).to_bytes(width,'little'));call=f'mirror_integer(&a,&b,{size})';calls+=4
 else:
  if row['a']>len(row['left'])or row['b']>len(row['right']):omitted.append(dict(case=name,reason='Unrepresentable logical extent; checked only as bounded Omega capacity rejection.'));continue
  values=[row['left'][:row['a']],row['right'][:row['b']]];want=values[0]+values[1];objects=[]
  for value in values:
   vec='vec!['+','.join(map(str,value))+']'
   if row['kind']=='string':
    try:bytes(value).decode('utf8')
    except UnicodeDecodeError:break
    objects.append(f'Object::String(String::from_utf8({vec}).unwrap())')
   else:objects.append(f'Object::Buffer({vec})')
  if len(objects)!=2:omitted.append(dict(case=name,reason='Invalid UTF-8 cannot construct Rust String; Omega ASCII/NUL-free profile returns encoding error.'));continue
  call=f'mirror_buffer(&a,&b,{size})'if row['kind']=='buffer'else'mirror_string(&a,&b)';calls+=4 if row['kind']=='buffer' else 2
 expected[name]=want
 lines.append(f'let a={objects[0]};let b={objects[1]};let actual={call};let mut conversions=a.to_buffer({size}).unwrap();conversions.extend(b.to_buffer({size}).unwrap());assert_eq!(actual,conversions);println!("{name}\\t{{:?}}",actual);')
lines.append('}');rust='\n'.join(lines);(HERE/'reference.rs').write_text(rust)
with tempfile.TemporaryDirectory(prefix='cathedral-byte-concat-rust-')as directory:
 work=Path(directory);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-byte-concat-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{UP}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n')
 lock=HERE/'reference.Cargo.lock'
 if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
 else:
  subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
 result=subprocess.run(['cargo','+nightly-2026-09-04','run','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-byte-concat-rust'],capture_output=True,text=True);assert result.returncode==0,result.stderr
 actual={name:json.loads(value)for name,value in(line.split('\t')for line in result.stdout.splitlines())};assert actual==expected
 record={'stage':'actual public Object::as_integer/to_integer/as_buffer/to_buffer calls plus explicitly labelled private append-expression mirrors; no actual do_concat/opcode/Store execution','pin':fixtures.PIN,'upstream_sha256':hashes,'private_do_concat_sha256':hashlib.sha256(private.encode()).hexdigest(),'mirror_observations':len(actual),'actual_public_calls':calls,'observed':actual,'omitted':omitted,'source_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [HERE/'reference.py',HERE/'reference.rs',HERE/'fixtures.py',HERE/'cases.json',lock]}}
 (HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');print('PASS',len(actual),'private-expression observations and',calls,'actual public conversion/access calls;',len(omitted),'explicit omissions')
