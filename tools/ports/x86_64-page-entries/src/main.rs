#![feature(step_trait)]
use core::iter::Step;
use x86_64::PhysAddr;
use x86_64::structures::paging::{PhysFrame,PageTableFlags,PageTableIndex,PageOffset};
use x86_64::structures::paging::page_table::{PageTableEntry,FrameError,PageTableLevel};
fn caught(f: impl FnOnce()->u64+std::panic::UnwindSafe)->Option<u64>{std::panic::catch_unwind(f).ok()}
fn main(){std::panic::set_hook(Box::new(|_|{}));
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),true);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4503599627366400),PageTableFlags::from_bits_retain(18442240474082185215));assert_eq!(e.addr().as_u64(),4503599627366400);assert_eq!(e.flags().bits(),18442240474082185215);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(3002399751577600),PageTableFlags::from_bits_retain(12294826982721456810));assert_eq!(e.addr().as_u64(),3002399751577600);assert_eq!(e.flags().bits(),12294826982721456810);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(1501199875788800),PageTableFlags::from_bits_retain(6147413491360728405));assert_eq!(e.addr().as_u64(),1501199875788800);assert_eq!(e.flags().bits(),6147413491360728405);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(1));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),1);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(2));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),2);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(4));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),4);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(8));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),8);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(16));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),16);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(32));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),32);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(64));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),64);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(128));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),128);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(256));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),256);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(512));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),512);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(1024));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),1024);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(2048));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),2048);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4096),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),4096);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(8192),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),8192);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(16384),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),16384);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(32768),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),32768);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(65536),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),65536);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(131072),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),131072);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(262144),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),262144);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(524288),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),524288);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(1048576),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),1048576);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(2097152),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),2097152);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4194304),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),4194304);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(8388608),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),8388608);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(16777216),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),16777216);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(33554432),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),33554432);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(67108864),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),67108864);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(134217728),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),134217728);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(268435456),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),268435456);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(536870912),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),536870912);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(1073741824),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),1073741824);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(2147483648),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),2147483648);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4294967296),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),4294967296);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(8589934592),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),8589934592);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(17179869184),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),17179869184);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(34359738368),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),34359738368);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(68719476736),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),68719476736);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(137438953472),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),137438953472);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(274877906944),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),274877906944);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(549755813888),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),549755813888);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(1099511627776),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),1099511627776);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(2199023255552),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),2199023255552);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4398046511104),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),4398046511104);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(8796093022208),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),8796093022208);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(17592186044416),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),17592186044416);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(35184372088832),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),35184372088832);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(70368744177664),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),70368744177664);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(140737488355328),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),140737488355328);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(281474976710656),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),281474976710656);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(562949953421312),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),562949953421312);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(1125899906842624),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),1125899906842624);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(2251799813685248),PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64(),2251799813685248);assert_eq!(e.flags().bits(),0);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(4503599627370496));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),4503599627370496);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(9007199254740992));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),9007199254740992);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(18014398509481984));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),18014398509481984);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(36028797018963968));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),36028797018963968);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(72057594037927936));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),72057594037927936);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(144115188075855872));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),144115188075855872);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(288230376151711744));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),288230376151711744);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(576460752303423488));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),576460752303423488);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(1152921504606846976));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),1152921504606846976);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(2305843009213693952));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),2305843009213693952);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(4611686018427387904));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),4611686018427387904);assert_eq!(e.is_unused(),false);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(9223372036854775808));assert_eq!(e.addr().as_u64(),0);assert_eq!(e.flags().bits(),9223372036854775808);assert_eq!(e.is_unused(),false);}
{assert_eq!(caught(||{let mut e=PageTableEntry::new(); e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(0)); e.addr().as_u64()|e.flags().bits()}),Some(0));}
{assert_eq!(caught(||{let mut e=PageTableEntry::new(); e.set_addr(PhysAddr::new(4096),PageTableFlags::from_bits_retain(3)); e.addr().as_u64()|e.flags().bits()}),Some(4099));}
{assert_eq!(caught(||{let mut e=PageTableEntry::new(); e.set_addr(PhysAddr::new(4503599627366400),PageTableFlags::from_bits_retain(9223372036854775935)); e.addr().as_u64()|e.flags().bits()}),Some(9227875636482142335));}
{assert_eq!(caught(||{let mut e=PageTableEntry::new(); e.set_addr(PhysAddr::new(4096),PageTableFlags::from_bits_retain(8193)); e.addr().as_u64()|e.flags().bits()}),Some(12289));}
{assert_eq!(caught(||{let mut e=PageTableEntry::new(); e.set_addr(PhysAddr::new(1),PageTableFlags::from_bits_retain(3)); e.addr().as_u64()|e.flags().bits()}),None);}
{assert_eq!(caught(||{let mut e=PageTableEntry::new(); e.set_addr(PhysAddr::new(4503599627370496),PageTableFlags::from_bits_retain(1)); e.addr().as_u64()|e.flags().bits()}),None);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(0));assert_eq!(e.frame().map(|p|p.start_address().as_u64()),Err(FrameError::FrameNotPresent));}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(128));assert_eq!(e.frame().map(|p|p.start_address().as_u64()),Err(FrameError::FrameNotPresent));}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(1));assert_eq!(e.frame().map(|p|p.start_address().as_u64()),Ok(0));}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(129));assert_eq!(e.frame().map(|p|p.start_address().as_u64()),Err(FrameError::HugeFrame));}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(305418240),PageTableFlags::from_bits_retain(1));assert_eq!(e.frame().map(|p|p.start_address().as_u64()),Ok(305418240));}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4503599627366400),PageTableFlags::from_bits_retain(18442240474082185215));assert_eq!(e.frame().map(|p|p.start_address().as_u64()),Err(FrameError::HugeFrame));}
{assert_eq!(caught(||{let mut v=PageTableEntry::new();v.set_frame(PhysFrame::containing_address(PhysAddr::new(0)),PageTableFlags::from_bits_retain(0));v.addr().as_u64()|v.flags().bits()}),Some(0));}
{assert_eq!(caught(||{let mut v=PageTableEntry::new();v.set_frame(PhysFrame::containing_address(PhysAddr::new(4096)),PageTableFlags::from_bits_retain(1));v.addr().as_u64()|v.flags().bits()}),Some(4097));}
{assert_eq!(caught(||{let mut v=PageTableEntry::new();v.set_frame(PhysFrame::containing_address(PhysAddr::new(4096)),PageTableFlags::from_bits_retain(129));v.addr().as_u64()|v.flags().bits()}),None);}
{assert_eq!(caught(||{let mut v=PageTableEntry::new();v.set_frame(PhysFrame::containing_address(PhysAddr::new(0)),PageTableFlags::from_bits_retain(18446744073709551615));v.addr().as_u64()|v.flags().bits()}),None);}
{assert_eq!(caught(||{let mut v=PageTableEntry::new();v.set_frame(PhysFrame::containing_address(PhysAddr::new(4503599627366400)),PageTableFlags::from_bits_retain(9223372036854775809));v.addr().as_u64()|v.flags().bits()}),Some(9227875636482142209));}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4503599627366400),PageTableFlags::from_bits_retain(18442240474082185215));e.set_flags(PageTableFlags::from_bits_retain(0));assert_eq!(e.addr().as_u64()|e.flags().bits(),4503599627366400);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4096),PageTableFlags::from_bits_retain(1));e.set_flags(PageTableFlags::from_bits_retain(3));assert_eq!(e.addr().as_u64()|e.flags().bits(),4099);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(4096),PageTableFlags::from_bits_retain(1));e.set_flags(PageTableFlags::from_bits_retain(8193));assert_eq!(e.addr().as_u64()|e.flags().bits(),12289);}
{let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(0),PageTableFlags::from_bits_retain(0));e.set_flags(PageTableFlags::from_bits_retain(18446744073709551615));assert_eq!(e.addr().as_u64()|e.flags().bits(),18446744073709551615);}
{assert_eq!(caught(||u64::from(PageTableIndex::new(0))),Some(0));}
{assert_eq!(u64::from(PageTableIndex::new_truncate(0)),0);}
{assert_eq!(caught(||u64::from(PageTableIndex::new(1))),Some(1));}
{assert_eq!(u64::from(PageTableIndex::new_truncate(1)),1);}
{assert_eq!(caught(||u64::from(PageTableIndex::new(511))),Some(511));}
{assert_eq!(u64::from(PageTableIndex::new_truncate(511)),511);}
{assert_eq!(caught(||u64::from(PageTableIndex::new(512))),None);}
{assert_eq!(u64::from(PageTableIndex::new_truncate(512)),0);}
{assert_eq!(caught(||u64::from(PageTableIndex::new(65535))),None);}
{assert_eq!(u64::from(PageTableIndex::new_truncate(65535)),511);}
{assert_eq!(caught(||u64::from(PageOffset::new(0))),Some(0));}
{assert_eq!(u64::from(PageOffset::new_truncate(0)),0);}
{assert_eq!(caught(||u64::from(PageOffset::new(1))),Some(1));}
{assert_eq!(u64::from(PageOffset::new_truncate(1)),1);}
{assert_eq!(caught(||u64::from(PageOffset::new(4095))),Some(4095));}
{assert_eq!(u64::from(PageOffset::new_truncate(4095)),4095);}
{assert_eq!(caught(||u64::from(PageOffset::new(4096))),None);}
{assert_eq!(u64::from(PageOffset::new_truncate(4096)),0);}
{assert_eq!(caught(||u64::from(PageOffset::new(65535))),None);}
{assert_eq!(u64::from(PageOffset::new_truncate(65535)),4095);}
{assert_eq!(Step::forward_checked(PageTableIndex::new(0),0).map(u64::from),Some(0));}
{let (v,o)=Step::forward_overflowing(PageTableIndex::new(0),0);assert_eq!((u64::from(v),o),(0,false));}
{assert_eq!(Step::forward_checked(PageTableIndex::new(0),511).map(u64::from),Some(511));}
{let (v,o)=Step::forward_overflowing(PageTableIndex::new(0),511);assert_eq!((u64::from(v),o),(511,false));}
{assert_eq!(Step::backward_checked(PageTableIndex::new(511),511).map(u64::from),Some(0));}
{let (v,o)=Step::backward_overflowing(PageTableIndex::new(511),511);assert_eq!((u64::from(v),o),(0,false));}
{assert_eq!(Step::forward_checked(PageTableIndex::new(511),1).map(u64::from),None);}
{let (v,o)=Step::forward_overflowing(PageTableIndex::new(511),1);assert_eq!((u64::from(v),o),(511,true));}
{assert_eq!(Step::backward_checked(PageTableIndex::new(0),1).map(u64::from),None);}
{let (v,o)=Step::backward_overflowing(PageTableIndex::new(0),1);assert_eq!((u64::from(v),o),(0,true));}
{assert_eq!(Step::forward_checked(PageTableIndex::new(511),18446744073709551615).map(u64::from),None);}
{let (v,o)=Step::forward_overflowing(PageTableIndex::new(511),18446744073709551615);assert_eq!((u64::from(v),o),(511,true));}
{assert_eq!(Step::backward_checked(PageTableIndex::new(511),18446744073709551615).map(u64::from),None);}
{let (v,o)=Step::backward_overflowing(PageTableIndex::new(511),18446744073709551615);assert_eq!((u64::from(v),o),(511,true));}
{assert_eq!(Step::steps_between(&PageTableIndex::new(0),&PageTableIndex::new(0)).1,Some(0));}
{assert_eq!(Step::steps_between(&PageTableIndex::new(0),&PageTableIndex::new(511)).1,Some(511));}
{assert_eq!(Step::steps_between(&PageTableIndex::new(511),&PageTableIndex::new(0)).1,None);}
{assert_eq!(Step::steps_between(&PageTableIndex::new(511),&PageTableIndex::new(511)).1,Some(0));}
{assert_eq!(PageTableLevel::One.next_lower_level().map(|v|v as u64),None);}
{assert_eq!(PageTableLevel::One.next_higher_level().map(|v|v as u64),Some(2));}
{assert_eq!(PageTableLevel::One.entry_address_space_alignment(),4096);}
{assert_eq!(PageTableLevel::One.table_address_space_alignment(),2097152);}
{assert_eq!(PageTableLevel::Two.next_lower_level().map(|v|v as u64),Some(1));}
{assert_eq!(PageTableLevel::Two.next_higher_level().map(|v|v as u64),Some(3));}
{assert_eq!(PageTableLevel::Two.entry_address_space_alignment(),2097152);}
{assert_eq!(PageTableLevel::Two.table_address_space_alignment(),1073741824);}
{assert_eq!(PageTableLevel::Three.next_lower_level().map(|v|v as u64),Some(2));}
{assert_eq!(PageTableLevel::Three.next_higher_level().map(|v|v as u64),Some(4));}
{assert_eq!(PageTableLevel::Three.entry_address_space_alignment(),1073741824);}
{assert_eq!(PageTableLevel::Three.table_address_space_alignment(),549755813888);}
{assert_eq!(PageTableLevel::Four.next_lower_level().map(|v|v as u64),Some(3));}
{assert_eq!(PageTableLevel::Four.next_higher_level().map(|v|v as u64),None);}
{assert_eq!(PageTableLevel::Four.entry_address_space_alignment(),549755813888);}
{assert_eq!(PageTableLevel::Four.table_address_space_alignment(),281474976710656);}
println!("143 actual pinned PTE/index/level witnesses passed");
}
