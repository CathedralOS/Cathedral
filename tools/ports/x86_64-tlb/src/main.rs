#![feature(step_trait)]
#![allow(dead_code)]
mod pinned;
use pinned::*;
use x86_64::{VirtAddr,structures::paging::{Page,Size4KiB,Size2MiB}};
fn main(){
 let offsets=descriptor_offsets();println!("descriptor_pcid_offset {}",offsets.0);println!("descriptor_address_offset {}",offsets.1);
 println!("pcid_size {}",core::mem::size_of::<Pcid>());println!("pcid_align {}",core::mem::align_of::<Pcid>());
 println!("descriptor_size {}",core::mem::size_of::<InvpcidDescriptor>());println!("descriptor_align {}",core::mem::align_of::<InvpcidDescriptor>());
 for value in [0u16,1,4095,4096,65535]{println!("pcid_{} {}",value,Pcid::new(value).map(|p|p.value() as u64).unwrap_or(65536));}
 let a:Page<Size4KiB>=Page::from_start_address(VirtAddr::new(0x1000)).unwrap();
 for maximum in [0,1,2,65535]{
  let end=a+3;let r=pinned_chunk(a,end,maximum).unwrap();println!("chunk_{} {} {} {}",maximum,r.0,r.1,r.2);
 }
 let page:Page<Size2MiB>=Page::from_start_address(VirtAddr::new(0xffff800000000000)).unwrap();
 let r=encode_broadcast(Some((page,65535)),Some(Pcid::new(4095).unwrap()),Some(65535),true,true,true);println!("broadcast_all {} {} {}",r.0,r.1,r.2);
 let (kind,desc)=prepare_invpcid(InvPcidCommand::Address(VirtAddr::new(0xffff800000001234),Pcid::new(4095).unwrap()));
 let bytes:[u8;16]=unsafe{core::mem::transmute(desc)};println!("invpcid_kind {}",kind);for (i,b) in bytes.iter().enumerate(){println!("invpcid_byte_{} {}",i,b);}
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn every_pcid(){for n in 0..=u16::MAX {let p=Pcid::new(n);assert_eq!(p.is_ok(),n<4096);if let Ok(p)=p{assert_eq!(p.value(),n);}}}
 #[test] fn all_invpcid_cases(){
  for (command,expected) in [(InvPcidCommand::Address(VirtAddr::new(0x1234),Pcid::new(9).unwrap()),(0,9,0x1234)),(InvPcidCommand::Single(Pcid::new(4095).unwrap()),(1,4095,0)),(InvPcidCommand::All,(2,0,0)),(InvPcidCommand::AllExceptGlobal,(3,0,0))]{let(k,d)=prepare_invpcid(command);let raw:[u64;2]=unsafe{core::mem::transmute(d)};assert_eq!((k,raw[0],raw[1]),expected);}
 }
 #[test] fn count_semantics_and_canonical_boundary(){
  let a:Page<Size4KiB>=Page::from_start_address(VirtAddr::new(0x1000)).unwrap();
  assert_eq!(pinned_chunk(a,a+1,15),Some((1,1,0x2000)));
  assert_eq!(pinned_chunk(a,a+3,0),Some((0,1,0x2000)));
  let low:Page<Size4KiB>=Page::from_start_address(VirtAddr::new(0x7ffffffff000)).unwrap();
  let high:Page<Size4KiB>=Page::from_start_address(VirtAddr::new(0xffff800000001000)).unwrap();
  assert_eq!(pinned_chunk(low,high,15),Some((1,1,0xffff800000000000)));
  assert_eq!(pinned_chunk(a,a,15),None);
 }
 #[test] fn broadcast_bit_combinations(){
  let a:Page<Size4KiB>=Page::from_start_address(VirtAddr::new(0x12345000)).unwrap();
  for bits in 0..64u64{
   let r=encode_broadcast(if bits&1!=0{Some((a,2))}else{None},if bits&2!=0{Some(Pcid::new(123).unwrap())}else{None},if bits&4!=0{Some(42)}else{None},bits&8!=0,bits&16!=0,bits&32!=0);
   assert_eq!(r.0,bits|if bits&1!=0{0x12345000}else{0});assert_eq!(r.1,if bits&1!=0{2}else{0});assert_eq!(r.2,if bits&2!=0{123<<16}else{0}|if bits&4!=0{42}else{0});
  }
 }
}
