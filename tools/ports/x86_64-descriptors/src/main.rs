#[cfg(test)] mod tests;
use core::mem::{size_of,align_of,offset_of};
use x86_64::structures::{DescriptorTablePointer,tss::TaskStateSegment,gdt::{DescriptorFlags,Entry}};

fn main() {
println!("DESCRIPTOR_ACCESSED={}",DescriptorFlags::ACCESSED.bits());
println!("DESCRIPTOR_WRITABLE={}",DescriptorFlags::WRITABLE.bits());
println!("DESCRIPTOR_CONFORMING={}",DescriptorFlags::CONFORMING.bits());
println!("DESCRIPTOR_EXECUTABLE={}",DescriptorFlags::EXECUTABLE.bits());
println!("DESCRIPTOR_USER_SEGMENT={}",DescriptorFlags::USER_SEGMENT.bits());
println!("DESCRIPTOR_DPL_RING_3={}",DescriptorFlags::DPL_RING_3.bits());
println!("DESCRIPTOR_PRESENT={}",DescriptorFlags::PRESENT.bits());
println!("DESCRIPTOR_AVAILABLE={}",DescriptorFlags::AVAILABLE.bits());
println!("DESCRIPTOR_LONG_MODE={}",DescriptorFlags::LONG_MODE.bits());
println!("DESCRIPTOR_DEFAULT_SIZE={}",DescriptorFlags::DEFAULT_SIZE.bits());
println!("DESCRIPTOR_GRANULARITY={}",DescriptorFlags::GRANULARITY.bits());
println!("DESCRIPTOR_LIMIT_0_15={}",DescriptorFlags::LIMIT_0_15.bits());
println!("DESCRIPTOR_LIMIT_16_19={}",DescriptorFlags::LIMIT_16_19.bits());
println!("DESCRIPTOR_BASE_0_23={}",DescriptorFlags::BASE_0_23.bits());
println!("DESCRIPTOR_BASE_24_31={}",DescriptorFlags::BASE_24_31.bits());
println!("DESCRIPTOR_KERNEL_DATA={}",DescriptorFlags::KERNEL_DATA.bits());
println!("DESCRIPTOR_KERNEL_CODE32={}",DescriptorFlags::KERNEL_CODE32.bits());
println!("DESCRIPTOR_KERNEL_CODE64={}",DescriptorFlags::KERNEL_CODE64.bits());
println!("DESCRIPTOR_USER_DATA={}",DescriptorFlags::USER_DATA.bits());
println!("DESCRIPTOR_USER_CODE32={}",DescriptorFlags::USER_CODE32.bits());
println!("DESCRIPTOR_USER_CODE64={}",DescriptorFlags::USER_CODE64.bits());
println!("DescriptorTablePointer.size={}",size_of::<DescriptorTablePointer>());
println!("DescriptorTablePointer.alignment={}",align_of::<DescriptorTablePointer>());
println!("DescriptorTablePointer.limit.offset={}",offset_of!(DescriptorTablePointer, limit));
println!("DescriptorTablePointer.base.offset={}",offset_of!(DescriptorTablePointer, base));
println!("TaskStateSegment.size={}",size_of::<TaskStateSegment>());
println!("TaskStateSegment.alignment={}",align_of::<TaskStateSegment>());
println!("TaskStateSegment.privilege_stack_table.offset={}",offset_of!(TaskStateSegment, privilege_stack_table));
println!("TaskStateSegment.interrupt_stack_table.offset={}",offset_of!(TaskStateSegment, interrupt_stack_table));
println!("TaskStateSegment.iomap_base.offset={}",offset_of!(TaskStateSegment, iomap_base));
println!("GdtEntry.size={}",size_of::<Entry>());
println!("GdtEntry.alignment={}",align_of::<Entry>());
}
