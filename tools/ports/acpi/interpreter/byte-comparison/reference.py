#!/usr/bin/env python3
"""Execute actual public pinned Object::aml_cmp; no private-body substitute."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;UP=ROOT/'reference_code/rust-osdev/acpi'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==fixtures.PIN
hashes={}
for name in ['src/aml/object.rs','src/aml/mod.rs','Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:
 content=subprocess.check_output(['git','show',fixtures.PIN+':'+name],cwd=UP);assert content==(UP/name).read_bytes();hashes[name]=hashlib.sha256(content).hexdigest()
lines=['use acpi::aml::object::Object;','fn main(){'];expected={};omitted=[]
for row in fixtures.cases():
 if row['a']!=len(row['left']) or row['b']!=len(row['right']):omitted.append(dict(case=row['name'],reason='Logical extent cannot be represented by initialized Rust buffer'));continue
 objects=[]
 for key in ['left','right']:
  data=row[key];literal='vec!['+','.join(map(str,data))+']'
  if row['profile']=='String':
   try:bytes(data).decode('utf8')
   except UnicodeDecodeError:break
   objects.append(f'Object::String(String::from_utf8({literal}).unwrap())')
  else:objects.append(f'Object::Buffer({literal})')
 if len(objects)!=2:omitted.append(dict(case=row['name'],reason='Invalid UTF8 cannot construct a Rust String'));continue
 order=(row['left']>row['right'])-(row['left']<row['right'])
 if row['profile']!='String':order=((row['a']>row['b'])-(row['a']<row['b']))or order
 expected[row['name']]=order
 lines.append(f'let a={objects[0]};let b={objects[1]};let order=match a.aml_cmp(&b).unwrap(){{std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1}};println!("{row["name"]}\\t{{order}}");')
lines.append('}')
with tempfile.TemporaryDirectory(prefix='cathedral-byte-comparison-rust-')as name:
 work=Path(name);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-byte-comparison-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{UP}"}}\n[[bin]]\nname="reference"\npath="main.rs"\n');(work/'main.rs').write_text('\n'.join(lines))
 lock=HERE/'reference.Cargo.lock'
 if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
 else:
  subprocess.run(['cargo','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],cwd=ROOT.parent/'Omega',check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
 result=subprocess.run(['cargo','run','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-byte-comparison-rust'],cwd=ROOT.parent/'Omega',capture_output=True,text=True)
 assert result.returncode==0,result.stderr
 actual={k:int(v)for k,v in(line.split('\t')for line in result.stdout.splitlines())};assert actual==expected
 record=dict(stage='actual public Object::aml_cmp calls on original owned values; no opcode execution',pin=fixtures.PIN,upstream_sha256=hashes,actual_public_calls=len(actual),omitted=omitted,observed=actual,source_sha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [HERE/'reference.py',HERE/'fixtures.py',HERE/'reference.Cargo.lock',HERE/'cases.json']})
 (HERE/'reference-verification.json').write_text(json.dumps(record,indent=2)+'\n')
 print('PASS',len(actual),'public Rust comparison observations;',len(omitted),'explicitly inapplicable inputs')
