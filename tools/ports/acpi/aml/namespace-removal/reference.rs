// SPDX-License-Identifier: MIT OR Apache-2.0
// Original actual public Namespace observations. No object-token acquisition,
// private implementation mirrors, AML execution, region access or hardware.
use acpi::{Handle,aml::{namespace::{Namespace,AmlName,NamespaceLevelKind},object::Object}};
use std::str::FromStr;
fn name(s:&str)->AmlName { AmlName::from_str(s).unwrap() }
fn observe(ns:&mut Namespace,label:&str) {
 let mut levels=Vec::new();let mut values=Vec::new();
 ns.traverse(|scope,level|{
  let prefix=scope.as_string();levels.push(prefix.clone());
  for (seg,(flags,_)) in &level.values {
   values.push(format!("{}{}{}:{}",prefix,if prefix=="\\"{""}else{"."},seg.as_str(),flags.is_alias()));
  }
  Ok(true)
 }).unwrap();
 levels.sort();values.sort();println!("{label}_levels\t{}",levels.join(","));println!("{label}_values\t{}",values.join(","));
}
fn main() {
 let args:Vec<String>=std::env::args().collect();let case=&args[1];let target=&args[2];
 // Handle0 names an actual host-owned mutex retained for the namespace lifetime.
 // Namespace traversal/removal never asks a handler to acquire this mutex.
 let _global_locks=[std::sync::Mutex::new(())];let mut ns=Namespace::new(Handle(0));
 let rows=[("\\AAAA",true,true),("\\AAAA.BBBB",true,true),("\\AAAA.BBBB.DATA",false,true),("\\AAAA.KEEP",false,true),("\\OTHR",true,true),("\\OTHR.DATA",false,true),("\\ONLY",false,true)];
 let objects:Vec<_>=(0..7).map(|i|Object::Integer(100+i).wrap()).collect();
 for (i,(path,mut level,mut value)) in rows.into_iter().enumerate() {
  if case=="leaf_level_only"&&i==3 {level=true;value=false}
  if case=="root_child_level_only"&&i==4 {value=false}
  if level {ns.add_level(name(path),NamespaceLevelKind::Device).unwrap()}
  if value {ns.insert(name(path),objects[i].clone()).unwrap()}
 }
 ns.create_alias(name("\\ALIA"),objects[2].clone()).unwrap();
 observe(&mut ns,"before");
 let result=ns.remove_level(name(target));println!("result\t{}",match result {Ok(())=>"ok".to_string(),Err(e)=>format!("error:{e:?}")});
 observe(&mut ns,"after");
 println!("same_path_object\t{}",ns.get(name("\\AAAA")).is_ok());println!("alias_survives\t{}",ns.get(name("\\ALIA")).is_ok());
}
