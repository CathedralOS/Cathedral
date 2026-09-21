// SPDX-License-Identifier: MIT OR Apache-2.0
// Original observations of the actual public pinned local Object method.
// Ordinary owned Rust values only: no interpreter, ObjectToken or services.
use acpi::aml::object::Object;
fn unhex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|pair|u8::from_str_radix(std::str::from_utf8(pair).unwrap(),16).unwrap()).collect()}
fn hex(bytes:&[u8])->String {bytes.iter().map(|b|format!("{b:02x}")).collect()}
fn main(){
 let a:Vec<String>=std::env::args().collect();let extent:usize=a[3].parse().unwrap();
 let source=match a[1].as_str(){"Integer"=>Object::Integer(a[2].parse().unwrap()),"String"=>Object::String(String::from_utf8(unhex(&a[2])).unwrap()),"Buffer"=>Object::Buffer(unhex(&a[2])),_=>panic!("unsupported fixture")};
 let mut target=Object::Buffer(vec![0xcc;extent]);
 match target.replace_with_implicit_casting(source){Ok(())=>match target {Object::Buffer(bytes)=>println!("result\tbuffer:{}:{}",bytes.len(),hex(&bytes)),_=>panic!("unexpected target type")},Err(e)=>println!("result\terror:{e:?}")}
}
