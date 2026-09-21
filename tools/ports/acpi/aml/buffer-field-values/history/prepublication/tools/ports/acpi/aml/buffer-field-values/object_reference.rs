// SPDX-License-Identifier: MIT OR Apache-2.0
// Calls actual public Object::read_buffer_field. No algorithm mirror/ObjectToken.
use acpi::aml::{IntegerSize,object::{Object,ReferenceKind}};
fn reference_kind(name:&str)->ReferenceKind {match name {"Named"=>ReferenceKind::Named,"Local"=>ReferenceKind::Local,"Arg"=>ReferenceKind::Arg,"RefOf"=>ReferenceKind::RefOf,"Index"=>ReferenceKind::Index,"Unresolved"=>ReferenceKind::Unresolved,_=>panic!("unknown reference kind")}}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));let a:Vec<String>=std::env::args().collect();
 let width=if a[1]=="32"{IntegerSize::FourBytes}else{IntegerSize::EightBytes};
 let bytes:Vec<u8>=a[3].as_bytes().chunks(2).map(|b|u8::from_str_radix(std::str::from_utf8(b).unwrap(),16).unwrap()).collect();
 let mut backing=if a[2]=="String"{Object::String(String::from_utf8(bytes).unwrap())}else{Object::Buffer(bytes)};
 if a[6]!="None"{backing=Object::Reference{kind:reference_kind(&a[6]),inner:backing.wrap()};}
 let mut value=Object::BufferField{buffer:backing.wrap(),offset:a[4].parse().unwrap(),length:a[5].parse().unwrap()};
 if a[7]!="None"{value=Object::Reference{kind:reference_kind(&a[7]),inner:value.wrap()};}
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||match value.read_buffer_field(width){
  Ok(Object::Integer(number))=>format!("integer:{number}"),
  Ok(Object::Buffer(bytes))=>format!("buffer:{}",bytes.iter().map(|b|format!("{b:02x}")).collect::<String>()),
  Ok(other)=>format!("unexpected:{:?}",other.typ()),Err(error)=>format!("error:{error:?}")
 }));println!("{}",result.unwrap_or_else(|_|"panic".into()));
}
