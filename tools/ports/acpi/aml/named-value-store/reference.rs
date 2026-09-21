// SPDX-License-Identifier: MIT OR Apache-2.0
// Original host-only probe. All observations use public pinned interpreter APIs.
use acpi::{Handler,Handle,PciAddress,RawPhysicalMapping};
use acpi::aml::{Interpreter,AmlError,namespace::AmlName,object::Object};
use acpi::address::{AddressSpace,GenericAddress,MappedGas};
use acpi::registers::{FixedRegisters,Pm1EventRegisterBlock,Pm1ControlRegisterBlock};
use std::{str::FromStr,sync::{Arc,atomic::{AtomicU32,Ordering}},panic::{catch_unwind,AssertUnwindSafe}};

#[derive(Clone,Default)]
struct Traps { mutexes:Arc<AtomicU32>, forbidden:Arc<AtomicU32> }
impl Traps { fn denied(&self)->! {self.forbidden.fetch_add(1,Ordering::SeqCst);panic!("forbidden host service")} }
macro_rules! trap {($name:ident($($arg:ident:$ty:ty),*) $(->$ret:ty)?)=>{fn $name(&self,$($arg:$ty),*) $(->$ret)? {$(let _=$arg;)*self.denied()}};}
impl Handler for Traps {
 unsafe fn map_physical_region<T>(&self,_address:usize,_size:usize)->RawPhysicalMapping<T>{self.denied()}
 unsafe fn unmap_physical_region<T>(&self,_region:RawPhysicalMapping<T>){self.denied()}
 trap!(read_u8(address:usize)->u8);trap!(read_u16(address:usize)->u16);trap!(read_u32(address:usize)->u32);trap!(read_u64(address:usize)->u64);
 trap!(write_u8(address:usize,value:u8));trap!(write_u16(address:usize,value:u16));trap!(write_u32(address:usize,value:u32));trap!(write_u64(address:usize,value:u64));
 trap!(read_io_u8(port:u16)->u8);trap!(read_io_u16(port:u16)->u16);trap!(read_io_u32(port:u16)->u32);
 trap!(write_io_u8(port:u16,value:u8));trap!(write_io_u16(port:u16,value:u16));trap!(write_io_u32(port:u16,value:u32));
 trap!(read_pci_u8(address:PciAddress,offset:u16)->u8);trap!(read_pci_u16(address:PciAddress,offset:u16)->u16);trap!(read_pci_u32(address:PciAddress,offset:u16)->u32);
 trap!(write_pci_u8(address:PciAddress,offset:u16,value:u8));trap!(write_pci_u16(address:PciAddress,offset:u16,value:u16));trap!(write_pci_u32(address:PciAddress,offset:u16,value:u32));
 trap!(nanos_since_boot()->u64);trap!(stall(microseconds:u64));trap!(sleep(milliseconds:u64));
 fn create_mutex(&self)->Handle {Handle(self.mutexes.fetch_add(1,Ordering::SeqCst))}
 trap!(acquire(mutex:Handle,timeout:u16)->Result<(),AmlError>);trap!(release(mutex:Handle));
 trap!(breakpoint());trap!(handle_debug(object:&Object));trap!(handle_fatal_error(kind:u8,code:u32,arg:u64));
}
fn interpreter(handler:Traps,revision:u8)->Interpreter<Traps> {
 // No physical target, mapping or I/O occurs. The actual pinned SystemIo
 // constructor stores numeric metadata with mapping=None. Every later access
 // is intercepted by Traps before it can perform a device operation.
 let gas=|address|unsafe{MappedGas::map_gas(GenericAddress{address_space:AddressSpace::SystemIo,bit_width:32,bit_offset:0,access_size:3,address},handler.clone()).unwrap()};
 let registers=Arc::new(FixedRegisters{pm1_event_registers:Pm1EventRegisterBlock{pm1_event_length:8,pm1a:gas(0x400),pm1b:None},pm1_control_registers:Pm1ControlRegisterBlock{pm1a:gas(0x600),pm1b:None}});
 Interpreter::new(handler,revision,registers,None)
}
fn hex(bytes:&[u8])->String{bytes.iter().map(|v|format!("{v:02x}")).collect()}
fn from_hex(text:&str)->Vec<u8>{(0..text.len()).step_by(2).map(|i|u8::from_str_radix(&text[i..i+2],16).unwrap()).collect()}
fn describe(value:&Object)->String {match value {Object::Integer(n)=>format!("integer:{n}"),Object::Buffer(v)=>format!("buffer:{}",hex(v)),Object::String(v)=>format!("string:{}",hex(v.as_bytes())),_=>format!("other:{:?}",value.typ())}}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));
 let args:Vec<String>=std::env::args().collect();let source=std::fs::read(&args[1]).unwrap();let revision:u8=args[2].parse().unwrap();
 let argument=match args[3].as_str(){"Integer"=>Object::Integer(args[4].parse().unwrap()),"Buffer"=>Object::Buffer(from_hex(&args[4])),"String"=>Object::String(String::from_utf8(from_hex(&args[4])).unwrap()),_=>panic!("source kind")}.wrap();
 let handler=Traps::default();let instance=interpreter(handler.clone(),revision);
 match instance.load_table(&source) {Ok(())=>println!("load\tok"),Err(e)=>{println!("load\terror:{e:?}");return}}
 let target=instance.namespace.lock().get(AmlName::from_str("\\TARG").unwrap()).unwrap();
 println!("before\t{}",describe(&target));println!("argument_before\t{}",describe(&argument));
 let result=catch_unwind(AssertUnwindSafe(||instance.evaluate(AmlName::from_str("\\MAIN").unwrap(),vec![argument.clone()])));
 match result {Ok(Ok(v))=>println!("result\t{}",describe(&v)),Ok(Err(e))=>println!("result\terror:{e:?}"),Err(_)=>println!("result\tpanic")}
 // Evaluation has returned or unwound: all temporary mutable borrows and the
 // interpreter's object-token lock are gone before immutable observations.
 let after=instance.namespace.lock().get(AmlName::from_str("\\TARG").unwrap()).unwrap();
 println!("after\t{}",describe(&after));println!("same_target\t{}",std::ptr::eq(&*target,&*after));
 println!("argument_after\t{}",describe(&argument));
 if args[5]=="alias" {
  let named=instance.namespace.lock().get(AmlName::from_str("\\SRCE").unwrap()).unwrap();let alias=instance.namespace.lock().get(AmlName::from_str("\\ALIA").unwrap()).unwrap();
  println!("alias_identity\t{}",std::ptr::eq(&*named,&*alias));println!("named_source_after\t{}",describe(&named));
  println!("source_distinct_target\t{}",!std::ptr::eq(&*named,&*after));
 }
 println!("forbidden_calls\t{}",handler.forbidden.load(Ordering::SeqCst));
 println!("created_mutexes\t{}",handler.mutexes.load(Ordering::SeqCst));
}
