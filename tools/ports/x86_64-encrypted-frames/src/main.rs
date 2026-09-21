// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual public pinned frame/address/iterator APIs in isolated detached profiles.
use std::{panic::{catch_unwind,AssertUnwindSafe},process::Command};
use x86_64::{PhysAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration as Config},paging::{PhysFrame,PageSize,Size4KiB,Size2MiB,Size1GiB}}};
const MAX:u64=(1<<52)-1;
fn frame<S:PageSize>(raw:u64)->PhysFrame<S>{PhysFrame::from_start_address(PhysAddr::new(raw)).unwrap()}
fn number<S:PageSize>(op:&str,a:u64,b:u64,n:u64,inclusive:bool,reverse:bool)->Option<u64>{
 Some(match op {
  "from_start"=>frame::<S>(a).start_address().as_u64(),
  "containing"=>PhysFrame::<S>::containing_address(PhysAddr::new(a)).start_address().as_u64(),
  "from_pfn"=>PhysFrame::<S>::try_from_pfn(a).ok()?.start_address().as_u64(),
  "pfn"=>frame::<S>(a).pfn(),
  "arithmetic"=>{let start=frame::<S>(a);let result=if reverse{start-n}else{start+n};result.start_address().as_u64()},
  "difference"=>frame::<S>(b)-frame::<S>(a),
  "range_count"|"range_bytes"|"range_at"=>{
   let start=frame::<S>(a);let end=frame::<S>(b);
   let count=if inclusive{PhysFrame::range_inclusive(start,end).len()}else{PhysFrame::range(start,end).len()};
   if op=="range_count"{count}else if op=="range_bytes"{
    if inclusive{PhysFrame::range_inclusive(start,end).size()}else{PhysFrame::range(start,end).size()}
   }else{
    if n>=count{return None;}
    let offset=if reverse{count-n-1}else{n};(start+offset).start_address().as_u64()
   }
  },
  _=>panic!("unknown fixture operation")
 })
}
fn emit<S:PageSize>(profile:&str,op:&str,a:u64,b:u64,n:u64,inclusive:bool,reverse:bool){
 let outcome=catch_unwind(||number::<S>(op,a,b,n,inclusive,reverse));
 let (ok,value,panicked)=match outcome{Ok(Some(v))=>(true,v,false),Ok(None)=>(false,0,false),Err(_)=>(false,0,true)};
 let strict=op=="arithmetic"&&n>u64::MAX/S::SIZE;let expected_ok=ok&&!strict;let expected_value=if expected_ok{value}else{0};
 println!("{{\"profile\":{profile:?},\"op\":{op:?},\"size\":{},\"start\":{a},\"end\":{b},\"index\":{n},\"inclusive\":{inclusive},\"reverse\":{reverse},\"rust_ok\":{ok},\"rust_value\":{value},\"rust_panicked\":{panicked},\"checked_multiplication\":{strict},\"ok\":{expected_ok},\"value\":{expected_value}}}",S::SIZE);
}
fn iterator<S:PageSize>(profile:&str,a:u64,b:u64,n:u64,inclusive:bool,reverse:bool){
 let mut out_start=a;let mut out_end=b;
 let result=catch_unwind(AssertUnwindSafe(||{
  let start=frame::<S>(a);let end=frame::<S>(b);
  if inclusive {
   let mut range=PhysFrame::range_inclusive(start,end);
   let selected=catch_unwind(AssertUnwindSafe(||if reverse{range.nth_back(n as usize)}else{range.nth(n as usize)}));
   out_start=range.start.start_address().as_u64();out_end=range.end.start_address().as_u64();selected
  }else{
   let mut range=PhysFrame::range(start,end);
   let selected=catch_unwind(AssertUnwindSafe(||if reverse{range.nth_back(n as usize)}else{range.nth(n as usize)}));
   out_start=range.start.start_address().as_u64();out_end=range.end.start_address().as_u64();selected
  }
 }));
 let (ok,value,failed)=match result{Ok(Ok(Some(v)))=>(true,v.start_address().as_u64(),false),Ok(Ok(None))=>(false,0,false),_=>(false,0,true)};
 println!("{{\"profile\":{profile:?},\"op\":\"iterator\",\"size\":{},\"start\":{a},\"end\":{b},\"index\":{n},\"inclusive\":{inclusive},\"reverse\":{reverse},\"ok\":{ok},\"value\":{value},\"failed\":{failed},\"out_start\":{out_start},\"out_end\":{out_end}}}",S::SIZE);
}
fn cases<S:PageSize>(profile:&str,bit:u64){
 let size=S::SIZE;let top=MAX&!(size-1);let valid_top=(MAX&!bit)&!(size-1);let boundary=if bit>=size&&bit<=MAX{bit-size}else{0};
 let mut addresses=vec![0,1,size,bit,bit|size,1<<47,1<<48,valid_top,MAX,u64::MAX];addresses.sort();addresses.dedup();
 for a in addresses {for op in ["from_start","containing","pfn"]{emit::<S>(profile,op,a,0,0,false,false);}}
 for pfn in [0,1,3,bit/size,(1<<48)/size,MAX/size,MAX/size+1,u64::MAX/size+1,u64::MAX]{emit::<S>(profile,"from_pfn",pfn,0,0,false,false);}
 let jump=if bit>0&&bit<(1<<51){bit*2/size}else{2};
 for (a,n)in [(0,0),(0,1),(size,1),(top,1),(valid_top,1),(0,u64::MAX/size+1),(size,u64::MAX),(boundary,1),(0,jump),(1<<47,0),(bit,0)] {
  for reverse in [false,true]{emit::<S>(profile,"arithmetic",a,0,n,false,reverse);}
 }
 for(a,b)in [(0,0),(0,size),(size,0),(boundary,top),(1<<47,1<<48)]{emit::<S>(profile,"difference",a,b,0,false,false);}
 let crossing=if bit>=size&&bit<(1<<51){bit*2}else{size*3};
 for(a,b)in [(0,0),(size,0),(0,size*3),(boundary,boundary),(0,crossing),(top,top),(valid_top,valid_top)]{
  for inclusive in [false,true]{
   emit::<S>(profile,"range_count",a,b,0,inclusive,false);emit::<S>(profile,"range_bytes",a,b,0,inclusive,false);
  }
 }
 // Actual cursor mutation, including caught panic ordering, exhaustion and band jumps.
 for(a,b,n)in [(0,0,0),(0,0,1),(0,size*3,0),(0,size*3,2),(0,size*3,3),(0,size*3,u64::MAX),(boundary,boundary,0),(top,top,0),(valid_top,valid_top,0),(0,crossing,crossing/size),(0,crossing,if bit>=size{bit/size}else{1})]{
  for inclusive in [false,true]{for reverse in [false,true]{
   emit::<S>(profile,"range_at",a,b,n,inclusive,reverse);iterator::<S>(profile,a,b,n,inclusive,reverse);
  }}
 }
}
fn profile(name:&str){
 let mut bit=0;
 if name!="disabled" {for part in name.split('_'){
  let position:u8=part[1..].parse().unwrap();bit=1u64<<position;
  let config=if part.starts_with('e'){Config::EncryptedBit(position)}else{Config::SharedBit(position)};
  // SAFETY: isolated process; no live tables, mappings or retained address/frame
  // values exist at reconfiguration. These are detached library arithmetic probes.
  unsafe{enable_memory_encryption(config)};
 }}
 cases::<Size4KiB>(name,bit);cases::<Size2MiB>(name,bit);cases::<Size1GiB>(name,bit);
}
fn main(){
 assert_eq!(usize::BITS,64,"Reference profile requires a 64-bit host usize");
 std::panic::set_hook(Box::new(|_|{}));
 if let Some(name)=std::env::args().nth(1){profile(&name);return;}
 for name in ["disabled","e0","e12","e21","e30","e47","s47","e48","e51","e52","e63","e47_s48","e47_s48_e47"]{
  let output=Command::new(std::env::current_exe().unwrap()).arg(name).output().unwrap();assert!(output.status.success());print!("{}",String::from_utf8(output.stdout).unwrap());
 }
}
