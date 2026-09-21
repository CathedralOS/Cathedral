#!/usr/bin/env python3
"""Actual public owned-object reads/clones and exact private copy_bits mirror."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];UP=ROOT/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
hashes={}
for name in ['src/aml/object.rs','src/aml/mod.rs','Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:
 data=subprocess.check_output(['git','show',PIN+':'+name],cwd=UP);assert data==(UP/name).read_bytes();hashes[name]=hashlib.sha256(data).hexdigest()
object_source=(UP/'src/aml/object.rs').read_text();mirror=object_source[object_source.index('pub(crate) fn copy_bits('):object_source.index('\n#[inline]\npub(crate) fn align_down')]
rust='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2018 Isaac Woods. Exact private copy_bits body below is a mirror;
// Object clone/read_buffer_field/to_integer calls use the real pinned crate.
// No forged ObjectToken, WrappedObject mutable access, AML opcode or live handler.
use acpi::aml::{IntegerSize,object::{Object,ReferenceKind}};
'''+mirror+'''
fn main(){
 let data:Vec<u8>=(0..256).map(|i|((i*29+7)&255)as u8).collect();
 for width in [4usize,8] {
  let size=if width==4 {IntegerSize::FourBytes}else{IntegerSize::EightBytes};
  for (label,start,count)in [("low",0,8),("cross",5,17),("native",0,width*8),("last",2040,8),("too_wide",0,width*8+1)] {
   let field=Object::BufferField {buffer:Object::Buffer(data.clone()).wrap(),offset:start,length:count};
   let read=field.read_buffer_field(size).unwrap();
   let kind=match &read {Object::Integer(_)=>"Integer",Object::Buffer(_)=>"Buffer",_=>panic!("kind")};
   let numeric=read.to_integer(size).unwrap();
   println!("{{\\"case\\":\\"field_read_{width}_{label}\\",\\"stage\\":\\"actual public read_buffer_field then to_integer\\",\\"kind\\":{kind:?},\\"number\\":{numeric}}}");
  }
  for (label,start,count,value)in [("cross",5,17,u64::MAX),("native",0,width*8,0x8877665544332211),("zero_pad",0,128,u64::MAX),("last",2040,8,0x51)] {
   let bytes=value.to_le_bytes();let mut output=data.clone();copy_bits(&bytes[..width],0,&mut output,start,count);
   println!("{{\\"case\\":\\"field_write_{width}_{label}\\",\\"stage\\":\\"exact private copy_bits body mirror\\",\\"bytes\\":{output:?}}}");
  }
 }
 let original=Object::Buffer(vec![65,66]);let mut copied=original.clone();
 if let Object::Buffer(ref mut bytes)=copied {bytes[0]=99;}else{panic!("copy type")};
 assert_eq!(original.as_buffer().unwrap(),[65,66]);assert_eq!(copied.as_buffer().unwrap(),[99,66]);
 println!("{{\\"case\\":\\"public_buffer_deep_clone\\",\\"independent\\":true}}");
 let original=Object::String(String::from("AB"));let mut copied=original.clone();
 if let Object::String(ref mut text)=copied {text.replace_range(0..1,"C");}else{panic!("copy type")};
 assert_eq!(original.to_buffer(IntegerSize::EightBytes).unwrap(),[65,66]);assert_eq!(copied.to_buffer(IntegerSize::EightBytes).unwrap(),[67,66]);
 println!("{{\\"case\\":\\"public_string_deep_clone\\",\\"independent\\":true}}");
 let backing=Object::Buffer(vec![65,66]).wrap();let alias=backing.clone();assert!(core::ptr::eq(&*alias,&*backing));
 let field=Object::BufferField {buffer:backing.clone(),offset:0,length:8};let copied=field.clone();
 if let Object::BufferField{buffer,..}=copied {assert!(core::ptr::eq(&*buffer,&*backing));}else{panic!("field clone")};
 let reference=Object::Reference {kind:ReferenceKind::RefOf,inner:backing.clone()};
 if let Object::Reference{inner,..}=reference.clone(){assert!(core::ptr::eq(&*inner,&*backing));}else{panic!("ref clone")};
 let package=Object::Package(vec![backing.clone()]);if let Object::Package(items)=package.clone(){assert!(core::ptr::eq(&*items[0],&*backing));}else{panic!("package clone")};
 println!("{{\\"case\\":\\"public_shared_identity_clones\\",\\"wrapped_reference_field_package_shared\\":true}}");
}
'''
(HERE/'reference.rs').write_text(rust)
with tempfile.TemporaryDirectory(prefix='cathedral-owned-bytes-rust-')as directory:
 work=Path(directory);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-owned-bytes-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{UP}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n');lock=HERE/'reference.Cargo.lock'
 if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
 else:
  subprocess.run(['cargo','+nightly-2026-09-04','generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
 result=subprocess.run(['cargo','+nightly-2026-09-04','run','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-owned-bytes-rust'],capture_output=True,text=True);assert result.returncode==0,result.stderr
rows=[json.loads(line)for line in result.stdout.splitlines()];pattern=[(i*29+7)&255 for i in range(256)]
for row in rows:
 if row['case'].startswith('field_read_'):
  _,_,width,label=row['case'].split('_',3);width=int(width);start,count={'low':(0,8),'cross':(5,17),'native':(0,width*8),'last':(2040,8),'too_wide':(0,width*8+1)}[label];expected=(int.from_bytes(bytes(pattern),'little')>>start)&((1<<min(count,width*8))-1);assert row['number']==expected,row
  row['omega_relation']='same numeric result; helper explicitly returns IntegerResult for count<=integer bits'if count<=width*8 else'Numeric-only Omega field read rejects wider count; generic Buffer result pending'
  row['pin_note']='Pinned read_buffer_field compares bit count with IntegerSize byte count, so some numeric fields are returned as Buffer; numeric helper uses ACPI bit width.'
 if row['case'].startswith('field_write_'):
  _,_,width,label=row['case'].split('_',3);width=int(width);start,count,number={'cross':(5,17,2**64-1),'native':(0,width*8,0x8877665544332211),'zero_pad':(0,128,2**64-1),'last':(2040,8,0x51)}[label];bits=int.from_bytes(bytes(pattern),'little');mask=((1<<count)-1)<<start;bits=(bits&~mask)|(((number&((1<<(width*8))-1))&((1<<count)-1))<<start);assert row['bytes']==list(bits.to_bytes(256,'little'))
record=dict(stage='actual public immutable Object read/clone/access observations plus exact labelled private copy_bits write mirror; no forged token, AML opcode or live firmware',pin=PIN,upstream_sha256=hashes,copy_bits_exact_sha256=hashlib.sha256(mirror.encode()).hexdigest(),observations=rows,source_sha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [HERE/'reference.py',HERE/'reference.rs',lock]})
(HERE/'reference-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');print('PASS',len(rows),'public/mirror observations')
