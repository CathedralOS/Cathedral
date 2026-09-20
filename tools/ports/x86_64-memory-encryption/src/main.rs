// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual pinned APIs in isolated processes. Configuration precedes construction
// of every address/PTE; no table or physical mapping is installed or accessed.
use std::{panic::{catch_unwind, AssertUnwindSafe},process::Command};
use x86_64::{PhysAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration as Config},paging::{page_table::PageTableEntry,PageTableFlags,PhysFrame}}};
fn json_value(value: Option<u64>)->String { value.map_or("null".to_string(),|v|v.to_string()) }
fn raw(entry: &PageTableEntry)->u64 { entry.addr().as_u64() | entry.flags().bits() }
fn profile(name: &str) {
    let mut bit=0;
    if name!="disabled" {
        for part in name.split('_') {
            let position:u8=part[1..].parse().unwrap(); bit=1u64<<position;
            let config=if part.starts_with('e'){Config::EncryptedBit(position)}else{Config::SharedBit(position)};
            // SAFETY: this isolated process has no existing PTE or physical
            // address value when reconfiguring, and creates no live mappings.
            // Positions outside actual CPU configurations are arithmetic probes.
            unsafe {enable_memory_encryption(config)};
        }
    }
    let mut words=vec![0,1,128,129,u64::MAX,bit,bit|1,bit|128,0x000f_ffff_ffff_f000,0xabcdef12345678];
    words.sort_unstable();words.dedup();
    print!("{{\"profile\":{name:?},\"observations\":[");
    for (i,word) in words.into_iter().enumerate() {
        let flags=PageTableFlags::from_bits_retain(word);
        let encrypted=flags.is_encrypted();
        let mut false_flags=flags;let clear=catch_unwind(AssertUnwindSafe(||{false_flags.set_encrypted(false);false_flags.bits()})).ok();
        let mut true_flags=flags;let set=catch_unwind(AssertUnwindSafe(||{true_flags.set_encrypted(true);true_flags.bits()})).ok();
        let mut entry=PageTableEntry::new(); entry.set_addr(PhysAddr::zero(),flags);
        let address=entry.addr().as_u64();let observed_flags=entry.flags().bits();
        let (frame_kind,frame)=match entry.frame(){Ok(f)=>(2,f.start_address().as_u64()),Err(x86_64::structures::paging::page_table::FrameError::FrameNotPresent)=>(0,0),Err(_)=>(1,0)};
        entry.set_flags(PageTableFlags::from_bits_retain(0xf000000000000007));let replaced=raw(&entry);
        let physical=PhysAddr::try_new(word).ok().map(|a|a.as_u64());let truncated=PhysAddr::new_truncate(word).as_u64();
        let set_addr=catch_unwind(AssertUnwindSafe(||{let mut out=PageTableEntry::new();out.set_addr(PhysAddr::new(word),PageTableFlags::from_bits_retain(0x8000000000000003));raw(&out)})).ok();
        let set_frame=catch_unwind(AssertUnwindSafe(||{let base=PhysFrame::from_start_address(PhysAddr::new(word)).unwrap();let mut out=PageTableEntry::new();out.set_frame(base,PageTableFlags::from_bits_retain(0x8000000000000003));raw(&out)})).ok();
        if i>0{print!(",");}
        print!("{{\"word\":{word},\"encrypted\":{encrypted},\"clear\":{},\"set\":{},\"address\":{address},\"flags\":{observed_flags},\"frame_kind\":{frame_kind},\"frame\":{frame},\"replaced\":{replaced},\"physical\":{},\"truncated\":{truncated},\"set_addr\":{},\"set_frame\":{}}}",json_value(clear),json_value(set),json_value(physical),json_value(set_addr),json_value(set_frame));
    }
    println!("]}}");
}
fn main(){
    std::panic::set_hook(Box::new(|_|{}));
    if let Some(name)=std::env::args().nth(1){profile(&name);return;}
    let mut profiles=vec!["disabled".to_string()];
    for position in 0..64 {profiles.push(format!("e{position}"));profiles.push(format!("s{position}"));}
    profiles.extend(["e47_s48","e47_s48_e47","e12_s12","s63_e0_s51"].map(str::to_owned));
    for name in &profiles {
        let output=Command::new(std::env::current_exe().unwrap()).arg(name).output().unwrap();assert!(output.status.success(),"profile {name}: {:?}",output.stderr);
        print!("{}",String::from_utf8(output.stdout).unwrap());
    }
    eprintln!("PASS {} isolated actual pinned configuration profiles",profiles.len());
}
