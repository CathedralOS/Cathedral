// SPDX-License-Identifier: MIT OR Apache-2.0
// Pinned pure register expressions only; never call CR3/MSR read/write methods.
use std::{panic::{catch_unwind},process::Command};
use x86_64::{PhysAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration as Config},paging::PhysFrame},registers::{control::Cr3Flags,model_specific::ApicBaseFlags}};
fn cr3(value:u64)->(u64,u16) {
 let addr = PhysAddr::new(value & 0x_000f_ffff_ffff_f000);
 let frame = PhysFrame::<x86_64::structures::paging::Size4KiB>::containing_address(addr);
 (frame.start_address().as_u64(),(value & 0xFFF) as u16)
}
fn apic(raw:u64)->u64 {
 let addr = PhysAddr::new_truncate(raw);
 let frame = PhysFrame::<x86_64::structures::paging::Size4KiB>::containing_address(addr);
 frame.start_address().as_u64()
}
fn cr3_operand(frame:PhysFrame,val:u16,top_bit:bool)->u64 {
 let addr = frame.start_address();
 let value = ((top_bit as u64) << 63) | addr.as_u64() | val as u64;
 value
}
fn apic_operand(frame:PhysFrame,flags:u64)->u64 {let addr=frame.start_address();flags | addr.as_u64()}
fn preserving(old_flags:u64,frame:PhysFrame,flags:ApicBaseFlags)->u64 {
 let reserved = old_flags & !(ApicBaseFlags::all().bits());
 let new_flags = reserved | flags.bits();
 apic_operand(frame,new_flags)
}
fn json(value:Option<u64>)->String {value.map_or("null".into(),|v|v.to_string())}
fn profile(name:&str) {
 let mut bit=0;
 if name!="disabled" {for part in name.split('_') {
  let pos:u8=part[1..].parse().unwrap();bit=1u64<<pos;
  let config=if part.starts_with('e'){Config::EncryptedBit(pos)}else{Config::SharedBit(pos)};
  // SAFETY: isolated process; configuration precedes every physical address.
  // These are numeric probes without live mappings or register access.
  unsafe{enable_memory_encryption(config)};
 }}
 let raws=[0,1,0x2000,0x12345abc,bit,bit|0x2001,1<<47,1<<48,1<<51,u64::MAX];
 for raw in raws {
  let observed=catch_unwind(||cr3(raw)).ok();
  let frame=catch_unwind(||PhysFrame::from_start_address(PhysAddr::new(raw)).ok()).ok().flatten();
  let low=0xfabc;let flags=Cr3Flags::from_bits_retain(0x8018);
  let aflags=ApicBaseFlags::from_bits_retain(0x80800);let old=0xabcdef12345000;
  let a=frame.map(|f|cr3_operand(f,low,false));let b=frame.map(|f|cr3_operand(f,low,true));
  let c=frame.map(|f|cr3_operand(f,flags.bits() as u16,false));let d=frame.map(|f|cr3_operand(f,0xabc,true));
  let e=frame.map(|f|apic_operand(f,0x8000000000010800));let f=frame.map(|f|preserving(old,f,aflags));
  println!("{{\"profile\":{name:?},\"raw\":{raw},\"cr3_frame\":{},\"cr3_low\":{},\"apic_frame\":{},\"apic_flags\":{},\"cr3_operand\":{},\"cr3_no_flush\":{},\"cr3_flags\":{},\"cr3_pcid\":{},\"apic_operand\":{},\"apic_preserving\":{}}}",json(observed.map(|v|v.0)),observed.map_or(0,|v|v.1),apic(raw),ApicBaseFlags::from_bits_truncate(raw).bits(),json(a),json(b),json(c),json(d),json(e),json(f));
 }
}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));
 if let Some(name)=std::env::args().nth(1){profile(&name);return;}
 for name in ["disabled","e0","e7","e12","e21","e47","s47","e51","s63","e47_s48_e47"] {
  let output=Command::new(std::env::current_exe().unwrap()).arg(name).output().unwrap();assert!(output.status.success());print!("{}",String::from_utf8(output.stdout).unwrap());
 }
}
