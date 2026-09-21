// SPDX-License-Identifier: MIT OR Apache-2.0
// Calls actual public pinned Object methods; no copied algorithm or ObjectToken.
use acpi::aml::{IntegerSize,object::{Object,ReferenceKind}};
fn hex(bytes:&[u8])->String {bytes.iter().map(|b|format!("{b:02x}")).collect()}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));let a:Vec<String>=std::env::args().collect();
 let width=if a[2]=="32" {IntegerSize::FourBytes}else{IntegerSize::EightBytes};
 let data:Vec<u8>=a[3].as_bytes().chunks(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect();
 let value=match a[1].as_str(){"Integer"=>Object::Integer(a[4].parse().unwrap()),"String"=>Object::String(String::from_utf8(data).unwrap()),"Buffer"=>Object::Buffer(data),"Field"=>Object::BufferField{buffer:Object::Buffer(data).wrap(),offset:a[4].parse().unwrap(),length:a[5].parse().unwrap()},"Reference"=>Object::Reference{kind:ReferenceKind::RefOf,inner:Object::Integer(42).wrap()},"Package"=>Object::Package(vec![]),_=>Object::Uninitialized};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
  if a[6]=="integer" {match value.to_integer(width){Ok(n)=>format!("integer:{n}"),Err(e)=>format!("error:{e:?}")}}
  else{match value.to_buffer(width){Ok(b)=>format!("buffer:{}",hex(&b)),Err(e)=>format!("error:{e:?}")}}
 }));println!("{}",result.unwrap_or_else(|_|"panic".into()));
}
