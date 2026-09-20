// SPDX-License-Identifier: MIT OR Apache-2.0
use bit_field::BitField;
struct Observed {ecx:u32,ebx:u32}
fn rdrand(cpuid:Observed)->bool { cpuid.ecx & (1 << 30) != 0 }
fn smap(cpuid:Observed)->bool { cpuid.ebx.get_bit(20) }
fn main(){for bit in 0..32 {let x=1u32<<bit;assert_eq!(rdrand(Observed{ecx:x,ebx:0}),bit==30);assert_eq!(smap(Observed{ecx:0,ebx:x}),bit==20);}for x in [0,u32::MAX]{assert_eq!(rdrand(Observed{ecx:x,ebx:x}),x!=0);assert_eq!(smap(Observed{ecx:x,ebx:x}),x!=0);}println!("PASS 68 exact pinned predicate observations; no CPUID execution");}
