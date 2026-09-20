#!/usr/bin/env python3
from pathlib import Path
import sys
CHECK="--check" in sys.argv
def emit(path,text):
 if CHECK:
  if not path.exists() or path.read_text()!=text:raise SystemExit("generated artifact differs: "+str(path))
 else:path.write_text(text)
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
source=(ROOT/'reference_code/rust-osdev/x86_64/src/registers/model_specific.rs').read_text()
def declaration(start):
 first=source.index('{',start);depth=1;end=first+1
 while depth:
  depth+=(source[end]=='{')-(source[end]=='}');end+=1
 return source[start:end]
write=declaration(source.index('pub fn write(\n            cs_sysret:'))
error=declaration(source.index('pub enum InvalidStarSegmentSelectors'))
tests=r'''
// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual upstream pure methods, plus unchanged STAR write body with a recording leaf.
use x86_64::registers::{control::PriorityClass,debug::*,segmentation::SegmentSelector};
use x86_64::PrivilegeLevel;
#[test]
fn exhaustive_codes_and_fields() {
    for value in 0..=255u8 {
        assert_eq!(PriorityClass::new(value).is_some(),(1..=15).contains(&value));
        assert_eq!(DebugAddressRegisterNumber::new(value).is_some(),value<4);
        assert_eq!(BreakpointCondition::from_bits(value as u64).is_some(),value<4);
        assert_eq!(BreakpointSize::from_bits(value as u64).is_some(),value<4);
        assert_eq!(BreakpointSize::new(value as usize).is_some(),[1,2,4,8].contains(&value));
    }
    for index in [0,1,8191] {
        for rpl in 0..4 {
            let selector=SegmentSelector::new(index,PrivilegeLevel::from_u16(rpl));
            assert_eq!(selector.0,(index<<3)|rpl);assert_eq!(selector.index(),index);
            assert_eq!(selector.rpl() as u16,rpl);
        }
    }
    let mut selector=SegmentSelector(0xffff);selector.set_rpl(PrivilegeLevel::Ring0);assert_eq!(selector.0,0xfffc);
    assert_eq!(Dr7Value::from_bits_truncate(u64::MAX).bits(),0xffff2bff);
    assert!(Dr7Value::from_bits(1<<10).is_none());assert!(Dr7Value::from_bits(1<<32).is_none());
    for n in 0..4u8 {
        assert_eq!(Dr6Flags::trap(DebugAddressRegisterNumber::new(n).unwrap()).bits(),1<<n);
        assert_eq!(Dr7Flags::local_breakpoint_enable(DebugAddressRegisterNumber::new(n).unwrap()).bits(),1<<(2*n));
        assert_eq!(Dr7Flags::global_breakpoint_enable(DebugAddressRegisterNumber::new(n).unwrap()).bits(),1<<(2*n+1));
        for condition in 0..4u64 {for size in 0..4u64 {
            let mut value=Dr7Value::from_bits(0x12342bff).unwrap();
            value.set_condition(DebugAddressRegisterNumber::new(n).unwrap(),BreakpointCondition::from_bits(condition).unwrap());
            value.set_size(DebugAddressRegisterNumber::new(n).unwrap(),BreakpointSize::from_bits(size).unwrap());
            let expected=(0x12342bff&!(15<<(16+4*n)))|(condition<<(16+4*n))|(size<<(18+4*n));
            assert_eq!(value.bits(),expected);
            assert_eq!(value.condition(DebugAddressRegisterNumber::new(n).unwrap()) as u64,condition);
            assert_eq!(value.size(DebugAddressRegisterNumber::new(n).unwrap()) as u64,size);
        }}
    }
    let flags=Dr7Flags::from_bits(3).unwrap();let mut value=Dr7Value::from_bits(0x12342bff).unwrap();
    value.remove_flags(flags);assert_eq!(value.bits(),0x12342bfc);
    value.insert_flags(flags);assert_eq!(value.bits(),0x12342bff);
    value.toggle_flags(flags);assert_eq!(value.bits(),0x12342bfc);
    value.set_flags(flags,true);assert_eq!(value.bits(),0x12342bff);
    value.set_flags(flags,false);assert_eq!(value.bits(),0x12342bfc);
}
thread_local! {static WRITTEN:std::cell::RefCell<Option<(u16,u16)>>=const {std::cell::RefCell::new(None)};}
struct Star;
impl Star {
    unsafe fn write_raw(sysret:u16,syscall:u16) {WRITTEN.with(|slot|*slot.borrow_mut()=Some((sysret,syscall)));}
'''+write+r'''
}
#[derive(Debug)]
'''+error+r'''
#[test]
fn star_reference_validation_and_recording() {
    Star::write(SegmentSelector(0x23),SegmentSelector(0x1b),SegmentSelector(8),SegmentSelector(16)).unwrap();
    WRITTEN.with(|slot|assert_eq!(*slot.borrow(),Some((0x13,8))));
    assert!(matches!(Star::write(SegmentSelector(0),SegmentSelector(0),SegmentSelector(8),SegmentSelector(16)),Err(InvalidStarSegmentSelectors::SysretOffset)));
    assert!(matches!(Star::write(SegmentSelector(0x23),SegmentSelector(0x1b),SegmentSelector(8),SegmentSelector(8)),Err(InvalidStarSegmentSelectors::SyscallOffset)));
    assert!(matches!(Star::write(SegmentSelector(0x20),SegmentSelector(0x18),SegmentSelector(8),SegmentSelector(16)),Err(InvalidStarSegmentSelectors::SysretPrivilegeLevel)));
    assert!(matches!(Star::write(SegmentSelector(0x23),SegmentSelector(0x1b),SegmentSelector(11),SegmentSelector(19)),Err(InvalidStarSegmentSelectors::SyscallPrivilegeLevel)));
}
#[test]
#[should_panic(expected="attempt to subtract with overflow")]
fn pinned_star_small_sysret_base_underflows_after_validation() {
    Star::write(SegmentSelector(11),SegmentSelector(3),SegmentSelector(8),SegmentSelector(16)).unwrap();
}
'''
emit(HERE/'src/tests.rs',tests)
p=HERE/'src/main.rs';s=p.read_text();decl='#[cfg(test)] mod tests;\n'
if decl not in s:emit(p,decl+s)
print('upstream STAR body extracted without algorithm changes; only write_raw replaced by recording leaf')
