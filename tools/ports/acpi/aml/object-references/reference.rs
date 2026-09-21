// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual public pinned WrappedObject operations; no ObjectToken or mutation.
use acpi::aml::{object::{Object,ReferenceKind,MethodFlags},namespace::AmlName};
fn kind(i:usize)->ReferenceKind {match i {0=>ReferenceKind::Named,1=>ReferenceKind::RefOf,2=>ReferenceKind::Local,3=>ReferenceKind::Arg,4=>ReferenceKind::Index,5=>ReferenceKind::Unresolved,_=>panic!("kind")}}
fn observe(name:&str,chain:&[usize],name_path:bool){
 let terminal=if name_path{Object::NamePath{name:AmlName::root(),scope:AmlName::root()}}else{Object::Integer(u64::MAX)};
 let mut objects=vec![terminal.wrap()];
 for &k in chain {let inner=objects.last().unwrap().clone();objects.push(Object::Reference{kind:kind(k),inner}.wrap());}
 for transparent in [false,true] {
  let object=objects.last().unwrap().clone();let result=if transparent{object.unwrap_transparent_reference()}else{object.unwrap_reference()};
  let selected=objects.iter().position(|candidate|core::ptr::eq(&**candidate,&*result)).expect("public unwrap retains existing identity");
  println!("{{\"name\":{name:?},\"chain\":{chain:?},\"name_path\":{name_path},\"transparent\":{transparent},\"selected\":{selected}}}");
 }
}
fn main(){
 observe("maximum_chain",&vec![0;63],false);observe("terminal",&[],false);observe("name_path",&[],true);
 for a in 0..6 {observe(&format!("single_{a}"),&[a],false);observe(&format!("named_terminal_{a}"),&[a],true);for b in 0..6 {observe(&format!("chain_{a}_{b}"),&[a,b],false);}}
 let first=Object::Integer(11).wrap();let second=Object::Reference{kind:ReferenceKind::RefOf,inner:first.clone()}.wrap();let third=Object::Integer(22).wrap();
 let originals=[first.clone(),second.clone(),third.clone()];let package=Object::Package(originals.to_vec()).wrap();
 if let Object::Package(ref elements)=*package {for index in [0u64,1,2,3,u64::MAX] {let selected=elements.get(index as usize).and_then(|item|originals.iter().position(|original|core::ptr::eq(&**item,&**original)));let found=selected.is_some();let id=selected.unwrap_or(0);println!("{{\"package_selection\":true,\"index\":{index},\"found\":{found},\"selected\":{id}}}");}}
 let copied=(*package).clone().wrap();assert!(!core::ptr::eq(&*package,&*copied));
 if let Object::Package(ref elements)=*copied {for (index,item)in elements.iter().enumerate(){assert!(core::ptr::eq(&**item,&*originals[index]));} println!("{{\"clone\":\"package\",\"fresh_wrapper\":true,\"shared_elements\":true,\"count\":{}}}",elements.len());}else{panic!("clone type")}
 for k in 0..6 {let reference=Object::Reference{kind:kind(k),inner:first.clone()}.wrap();let copied=(*reference).clone().wrap();let Object::Reference{kind:observed,ref inner}=*copied else{panic!("clone type")};assert_eq!(observed,kind(k));assert!(core::ptr::eq(&**inner,&*first));assert!(!core::ptr::eq(&*reference,&*copied));println!("{{\"clone\":\"reference\",\"kind\":{k},\"fresh_wrapper\":true,\"shared_target\":true}}");}
 let method=Object::Method{code:vec![0xa4,0x0a,7],flags:MethodFlags(0)};let Object::Method{code,flags}=method.clone()else{panic!("clone type")};assert_eq!(code,[0xa4,0x0a,7]);assert_eq!(flags.0,0);println!("{{\"clone\":\"method\",\"code_and_flags_preserved\":true}}");
 let alias=first.clone();assert!(core::ptr::eq(&*first,&*alias));println!("{{\"clone\":\"wrapped\",\"same_identity\":true}}");
}
