// SPDX-License-Identifier: MIT OR Apache-2.0
// Original public API observations; no copied private body or live handler.
use acpi::aml::namespace::{AmlName,NameSeg};
use std::{str::FromStr,panic::catch_unwind};
fn hex(bytes:&[u8])->String {bytes.iter().map(|x|format!("{x:02x}")).collect()}
fn show(name:&str,value:Result<AmlName,acpi::aml::AmlError>) {
 match value {Ok(v)=>println!("{name}\tok:{}",hex(v.as_string().as_bytes())),Err(e)=>println!("{name}\terror:{e:?}")}
}
fn main() {
 std::panic::set_hook(Box::new(|_|{}));
 let args:Vec<String>=std::env::args().collect();
 let bytes:Vec<u8>=args[2].as_bytes().chunks(2).map(|x|u8::from_str_radix(std::str::from_utf8(x).unwrap(),16).unwrap()).collect();
 let text=std::str::from_utf8(&bytes).unwrap();
 let result=catch_unwind(||{
  if args[1]=="segment" {
   match NameSeg::from_str(text){Ok(v)=>println!("parsed\tok:{}",hex(v.as_str().as_bytes())),Err(e)=>println!("parsed\terror:{e:?}")};
  } else {
   match AmlName::from_str(text) {
    Err(e)=>println!("parsed\terror:{e:?}"),
    Ok(v)=>{
     println!("parsed\tok:{}",hex(v.as_string().as_bytes()));
     println!("absolute\t{}",v.is_absolute());println!("normal\t{}",v.is_normal());println!("search\t{}",v.search_rules_apply());
     show("normalized",v.clone().normalize());show("parent",v.parent());
     show("resolved",v.resolve(&AmlName::from_str("\\_SB.PCI0.DEV0").unwrap()));
    }
   }
  }
 });
 if result.is_err(){println!("panic\ttrue")}
}
