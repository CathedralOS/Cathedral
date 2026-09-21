#!/usr/bin/env python3
"""Call public pinned Object::to_integer; separately compare a labelled formatter mirror."""
import hashlib,json,shutil,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;UP=ROOT/'reference_code/rust-osdev/acpi'
def main():
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==fixtures.PIN
 pin_files=['src/aml/object.rs','src/aml/mod.rs','Cargo.toml','LICENCE-MIT','LICENCE-APACHE']
 hashes={}
 for name in pin_files:
  data=subprocess.check_output(['git','show',fixtures.PIN+':'+name],cwd=UP);assert data==(UP/name).read_bytes();hashes[name]=hashlib.sha256(data).hexdigest()
 upstream=(UP/'src/aml/mod.rs').read_text()
 for fragment in ['Object::String(value.clone())','Object::String(value.to_string())','alloc::format!("{value:#X}")','alloc::format!("{byte},")','alloc::format!("{byte:#04X},")','string.push_str(&as_str);','string.pop();']:assert fragment in upstream,fragment
 statements=[];expected={};omitted=[]
 for row in fixtures.cases():
  name=row['name']
  if row['kind']=='parse':
   if'pin_error'not in row:omitted.append({'case':name,'reason':'logical input extent exceeds physical initialized storage; local API rejection only'});continue
   try:bytes(row['data']).decode('utf8')
   except UnicodeDecodeError:omitted.append({'case':name,'reason':'invalid UTF-8 cannot construct a Rust String'});continue
   size='FourBytes'if row['width']==32 else'EightBytes';bytes_literal=','.join(map(str,row['data']))
   statements.append(f'let object=Object::String(String::from_utf8(vec![{bytes_literal}]).unwrap()); match object.to_integer(IntegerSize::{size}) {{ Ok(value)=>println!("{name}\\tparse\\t0\\t{{value}}"),Err(_)=>println!("{name}\\tparse\\t3\\t0") }};')
   expected[name]=['parse',str(row['pin_error']),str(row['pin_value']if not row['pin_error']else 0)]
  else:
   if row['kind']=='buffer'and row['length']!=len(row['data']):omitted.append({'case':name,'reason':'oversized logical extent has no matching initialized Rust buffer'});continue
   obj=f'Object::Integer({row["value"]})'if row['kind']=='integer'else'Object::Buffer(vec!['+','.join(map(str,row['data']))+'])'
   statements.append(f'println!("{name}\\tformat-mirror\\t{{}}",hexadecimal(&format_mirror(&{obj},{str(row["format"]=="Hexadecimal").lower()})));')
   values=[row['value']]if row['kind']=='integer'else row['data'];text=','.join(str(value)if row['format']=='Decimal'else(f'0x{value:X}'if row['kind']=='integer'else f'0x{value:02X}')for value in values)
   expected[name]=['format-mirror',text.encode().hex()]
 with tempfile.TemporaryDirectory(prefix='cathedral-acpi-string-numbers-rust-')as directory:
  work=Path(directory);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-acpi-string-number-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{UP}"}}\n[[bin]]\nname="reference"\npath="main.rs"\n')
  (work/'main.rs').write_text((HERE/'reference.rs').read_text()+'\nfn main(){\n'+'\n'.join(statements)+'\n}\n')
  cargo=shutil.which('mbx')or'cargo';lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   subprocess.run([cargo,'generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],cwd=ROOT.parent/'Omega',check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  run=subprocess.run([cargo,'run','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-acpi-string-numbers-reference','--bin','reference'],cwd=ROOT.parent/'Omega',capture_output=True,text=True)
  if run.returncode:raise SystemExit(run.stderr)
  observed={line.split('\t')[0]:line.split('\t')[1:]for line in run.stdout.splitlines()}
  assert observed==expected,[(k,expected[k],observed.get(k))for k in expected if expected[k]!=observed.get(k)]
  sources={p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in [HERE/'reference.py',HERE/'reference.rs',HERE/'reference.Cargo.lock',HERE/'fixtures.py',HERE/'cases.json']}
  record={'stage':'host Rust public Object::to_integer and explicitly labelled private formatter pure-body mirror; no AML opcode/target execution','upstream_revision':fixtures.PIN,'upstream_sha256':hashes,'source_sha256':sources,'rustc':subprocess.check_output(['rustc','--version'],cwd=ROOT.parent/'Omega',text=True).strip(),'actual_public_method_count':sum(v[0]=='parse'for v in observed.values()),'formatter_mirror_count':sum(v[0]=='format-mirror'for v in observed.values()),'omitted':omitted,'observed':observed}
  (HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 print('PASS',record['actual_public_method_count'],'actual pinned Object::to_integer calls;',record['formatter_mirror_count'],'labelled formatter mirrors;',len(omitted),'explicitly inapplicable cases')
if __name__=='__main__':main()
