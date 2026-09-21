// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual public write_buffer_field with the token created by Interpreter::new.
// Trap constructor adapted from Cathedral aml-public-execution; no fabricated token.
use acpi::{Handler,Handle,PciAddress,RawPhysicalMapping};
use acpi::aml::{Interpreter,AmlError,object::{Object,ReferenceKind}};
use acpi::address::{AddressSpace,GenericAddress,MappedGas};
use acpi::registers::{FixedRegisters,Pm1EventRegisterBlock,Pm1ControlRegisterBlock};
use std::{sync::{Arc,atomic::{AtomicU32,Ordering}},panic::{catch_unwind,AssertUnwindSafe}};

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
 // No physical backing, mapping or I/O occurs. The actual pinned SystemIo
 // constructor stores numeric metadata with mapping=None. Every later access
 // is intercepted by Traps before it can perform a device operation.
 let gas=|address|unsafe{MappedGas::map_gas(GenericAddress{address_space:AddressSpace::SystemIo,bit_width:32,bit_offset:0,access_size:3,address},handler.clone()).unwrap()};
 let registers=Arc::new(FixedRegisters{pm1_event_registers:Pm1EventRegisterBlock{pm1_event_length:8,pm1a:gas(0x400),pm1b:None},pm1_control_registers:Pm1ControlRegisterBlock{pm1a:gas(0x600),pm1b:None}});
 Interpreter::new(handler,revision,registers,None)
}
fn kind(name:&str)->ReferenceKind {match name {"Named"=>ReferenceKind::Named,"Local"=>ReferenceKind::Local,"Arg"=>ReferenceKind::Arg,"RefOf"=>ReferenceKind::RefOf,"Index"=>ReferenceKind::Index,"Unresolved"=>ReferenceKind::Unresolved,_=>panic!("unknown reference kind")}}
fn unhex(text:&str)->Vec<u8>{text.as_bytes().chunks(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn hex(bytes:&[u8])->String{bytes.iter().map(|b|format!("{b:02x}")).collect()}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));let a:Vec<String>=std::env::args().collect();
 let data=unhex(&a[2]);let payload=unhex(&a[5]);
 let leaf=if a[1]=="String"{Object::String(String::from_utf8(data).unwrap()).wrap()}else{Object::Buffer(data).wrap()};
 let backing=if a[6]=="None"{leaf.clone()}else{Object::Reference{kind:kind(&a[6]),inner:leaf.clone()}.wrap()};
 let mut field=Object::BufferField{buffer:backing,offset:a[3].parse().unwrap(),length:a[4].parse().unwrap()};
 if a[7]!="None"{field=Object::Reference{kind:kind(&a[7]),inner:field.wrap()};}
 let traps=Traps::default();let interpreter=interpreter(traps.clone(),2);
 let result=catch_unwind(AssertUnwindSafe(||{
  // This is the interpreter-created token's ordinary host Spinlock. It is
  // unrelated to the ACPI firmware GlobalLock and invokes no Handler lock.
  let token=interpreter.object_token.lock();
  let outcome=field.write_buffer_field(&payload,&token);
  drop(token);outcome
 }));
 // No backing reference was borrowed during the mutable operation; token guard
 // has been dropped before this immutable observation. No gain_mut is called.
 let after=match &*leaf {Object::Buffer(bytes)=>hex(bytes),Object::String(value)=>hex(value.as_bytes()),_=>panic!("wrong backing")};
 let outcome=match result{Ok(Ok(()))=>"ok".into(),Ok(Err(error))=>format!("error:{error:?}"),Err(_)=>"panic".into()};
 println!("result\t{outcome}\nafter\t{after}\nforbidden_calls\t{}\ncreated_mutexes\t{}",traps.forbidden.load(Ordering::SeqCst),traps.mutexes.load(Ordering::SeqCst));
}
