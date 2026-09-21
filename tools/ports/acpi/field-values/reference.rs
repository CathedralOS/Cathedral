// SPDX-License-Identifier: MIT OR Apache-2.0
// Original public Field observations. SystemMemory callbacks operate only on
// an initialized Vec and log numeric offsets; every other service is trapped.
use acpi::{Handler,Handle,PciAddress,RawPhysicalMapping};
use acpi::aml::{Interpreter,AmlError,namespace::AmlName,object::Object};
use acpi::address::{AddressSpace,GenericAddress,MappedGas};
use acpi::registers::{FixedRegisters,Pm1EventRegisterBlock,Pm1ControlRegisterBlock};
use std::{str::FromStr,sync::{Arc,atomic::{AtomicU32,Ordering}},panic::{catch_unwind,AssertUnwindSafe}};

#[derive(Clone,Default)]
struct Traps { mutexes:Arc<AtomicU32>, forbidden:Arc<AtomicU32>, memory:Arc<std::sync::Mutex<Vec<u8>>>, log:Arc<std::sync::Mutex<Vec<String>>> }
impl Traps { fn denied(&self)->! {self.forbidden.fetch_add(1,Ordering::SeqCst);panic!("forbidden host service")} }
macro_rules! trap {($name:ident($($arg:ident:$ty:ty),*) $(->$ret:ty)?)=>{fn $name(&self,$($arg:$ty),*) $(->$ret)? {$(let _=$arg;)*self.denied()}};}
impl Traps {
 fn read(&self,address:usize,width:usize)->u64 {
  let offset=address.checked_sub(0x1000).expect("synthetic base");
  let memory=self.memory.lock().unwrap();let bytes=&memory[offset..offset+width];let mut raw=[0u8;8];raw[..width].copy_from_slice(bytes);let value=u64::from_le_bytes(raw);
  self.log.lock().unwrap().push(format!("read,{offset},{width},{value}"));value
 }

}
impl Handler for Traps {
 unsafe fn map_physical_region<T>(&self,_address:usize,_size:usize)->RawPhysicalMapping<T>{self.denied()}
 unsafe fn unmap_physical_region<T>(&self,_region:RawPhysicalMapping<T>){self.denied()}
 fn read_u8(&self,a:usize)->u8 {self.read(a,1) as u8}
 fn read_u16(&self,a:usize)->u16 {self.read(a,2) as u16}
 fn read_u32(&self,a:usize)->u32 {self.read(a,4) as u32}
 fn read_u64(&self,a:usize)->u64 {self.read(a,8)}
 trap!(write_u8(address:usize,value:u8));
 trap!(write_u16(address:usize,value:u16));
 trap!(write_u32(address:usize,value:u32));
 trap!(write_u64(address:usize,value:u64));
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
fn hex(bytes:&[u8])->String {bytes.iter().map(|x|format!("{x:02x}")).collect()}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));
 let args:Vec<String>=std::env::args().collect();let source=std::fs::read(&args[1]).unwrap();let revision:u8=args[2].parse().unwrap();
 let pattern:u8=args[3].parse().unwrap();let handler=Traps::default();*handler.memory.lock().unwrap()=(0..512).map(|i|match pattern {1=>0,2=>255,3=>((i*91+0xa6)&255)as u8,_=>((i*37+11)&255)as u8}).collect();
 let instance=interpreter(handler.clone(),revision);
 let run=catch_unwind(AssertUnwindSafe(||{
  match instance.load_table(&source){Ok(())=>println!("load\tok"),Err(e)=>{println!("load\terror:{e:?}");return}}
  match instance.evaluate(AmlName::from_str("\\MAIN").unwrap(),vec![]) {
   Ok(value)=>match &*value {Object::Integer(n)=>println!("result\tinteger:{n}"),Object::Buffer(bytes)=>println!("result\tbuffer:{}:{}",bytes.len(),hex(bytes)),other=>println!("result\tother:{:?}",other.typ())},
   Err(e)=>println!("result\terror:{e:?}")
  }
 }));
 if run.is_err(){println!("panic\ttrue")}
 println!("accesses\t{}",handler.log.lock().unwrap().join(";"));
 println!("memory\t{}",hex(&handler.memory.lock().unwrap()));
 println!("forbidden_calls\t{}",handler.forbidden.load(Ordering::SeqCst));
 println!("created_mutexes\t{}",handler.mutexes.load(Ordering::SeqCst));
}
