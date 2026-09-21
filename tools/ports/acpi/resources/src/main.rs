// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual pinned public resource parser; no private mirror and no ACPI handlers.
use acpi::aml::{object::Object,resource::*};
fn normalized(value:Resource)->Vec<u64>{match value{
 Resource::Irq(i)=>{let mut out=vec![1,i.is_consumer as u64,(i.trigger==InterruptTrigger::Edge)as u64,(i.polarity==InterruptPolarity::ActiveLow)as u64,i.is_shared as u64,i.is_wake_capable as u64,i.irqs.len()as u64];out.extend(i.irqs.into_iter().map(u64::from));out},
 Resource::Dma(d)=>vec![2,d.channel_mask as u64,match d.supported_speeds{DMASupportedSpeed::CompatibilityMode=>0,DMASupportedSpeed::TypeA=>1,DMASupportedSpeed::TypeB=>2,DMASupportedSpeed::TypeF=>3},match d.transfer_type_preference{DMATransferTypePreference::_8BitOnly=>0,DMATransferTypePreference::_8And16Bit=>1,DMATransferTypePreference::_16Bit=>2},d.is_bus_master as u64],
 Resource::IOPort(i)=>vec![3,i.decodes_full_address as u64,i.memory_range.0 as u64,i.memory_range.1 as u64,i.base_alignment as u64,i.range_length as u64],
 Resource::MemoryRange(MemoryRangeDescriptor::FixedLocation{is_writable,base_address,range_length})=>vec![4,is_writable as u64,base_address as u64,range_length as u64],
 Resource::AddressSpace(a)=>vec![5,match a.resource_type{AddressSpaceResourceType::MemoryRange=>0,AddressSpaceResourceType::IORange=>1,AddressSpaceResourceType::BusNumberRange=>2},a.is_maximum_address_fixed as u64,a.is_minimum_address_fixed as u64,(a.decode_type==AddressSpaceDecodeType::Subtractive)as u64,a.granularity,a.address_range.0,a.address_range.1,a.translation_offset,a.length],
 _=>vec![99]
}}
fn observe(name:&str,bytes:&[u8]){
 let result=std::panic::catch_unwind(||resource_descriptor_list(Object::Buffer(bytes.to_vec()).wrap()));
 match result {Ok(Ok(items))=>println!("{{\"name\":\"{}\",\"result\":\"ok\",\"resources\":{:?}}}",name,items.into_iter().map(normalized).collect::<Vec<_>>()),Ok(Err(_))=>println!("{{\"name\":\"{}\",\"result\":\"error\"}}",name),Err(_)=>println!("{{\"name\":\"{}\",\"result\":\"panic\"}}",name)}
}
fn main(){std::panic::set_hook(Box::new(|_|{}));
observe("irq-default",&[34, 64, 0]);
observe("irq-empty",&[34, 0, 0]);
observe("irq-all",&[34, 255, 255]);
observe("irq-flags-1",&[35, 129, 128, 1]);
observe("irq-flags-8",&[35, 129, 128, 8]);
observe("irq-flags-17",&[35, 129, 128, 17]);
observe("irq-flags-24",&[35, 129, 128, 24]);
observe("irq-flags-33",&[35, 129, 128, 33]);
observe("irq-flags-40",&[35, 129, 128, 40]);
observe("irq-flags-49",&[35, 129, 128, 49]);
observe("irq-flags-56",&[35, 129, 128, 56]);
observe("dma-0-0",&[42, 132, 4]);
observe("dma-1-1",&[42, 132, 37]);
observe("dma-2-2",&[42, 132, 70]);
observe("dma-3-0",&[42, 132, 100]);
observe("io",&[71, 1, 248, 3, 248, 3, 8, 8]);
observe("io-zero",&[71, 0, 0, 0, 255, 255, 0, 0]);
observe("fixed-memory",&[134, 9, 0, 1, 0, 80, 52, 18, 0, 16, 0, 0]);
observe("fixed-memory-raw-max",&[134, 9, 0, 254, 255, 255, 255, 255, 255, 255, 255, 255]);
observe("address-2",&[136, 13, 0, 2, 12, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1]);
observe("address-max-2",&[136, 13, 0, 0, 2, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
observe("address-4",&[135, 23, 0, 1, 12, 0, 0, 0, 0, 0, 0, 1, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0]);
observe("address-max-4",&[135, 23, 0, 0, 2, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
observe("address-8",&[138, 43, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
observe("address-max-8",&[138, 43, 0, 0, 2, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
observe("address-source",&[136, 24, 0, 0, 12, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1, 7, 92, 95, 83, 66, 46, 76, 78, 75, 65, 0]);
observe("address-empty-source",&[135, 25, 0, 0, 12, 0, 0, 0, 0, 0, 0, 1, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
observe("extended-one",&[137, 6, 0, 31, 1, 17, 0, 0, 0]);
observe("extended-many",&[137, 14, 0, 0, 3, 0, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]);
observe("extended-source",&[137, 17, 0, 2, 2, 16, 0, 0, 0, 17, 0, 0, 0, 3, 92, 76, 78, 75, 65, 0]);
observe("unsupported-small-6",&[48]);
observe("unsupported-small-7",&[56]);
observe("unsupported-small-9",&[72]);
observe("unsupported-small-10",&[80]);
observe("unsupported-small-14",&[112]);
observe("unsupported-large-1",&[129, 0, 0]);
observe("unsupported-large-2",&[130, 0, 0]);
observe("unsupported-large-4",&[132, 0, 0]);
observe("unsupported-large-5",&[133, 0, 0]);
observe("unsupported-large-11",&[139, 0, 0]);
observe("unsupported-large-12",&[140, 0, 0]);
observe("unsupported-large-13",&[141, 0, 0]);
observe("unsupported-large-14",&[142, 0, 0]);
observe("unsupported-large-15",&[143, 0, 0]);
observe("unsupported-large-16",&[144, 0, 0]);
observe("unsupported-large-17",&[145, 0, 0]);
observe("unsupported-large-18",&[146, 0, 0]);
observe("unsupported-large-19",&[147, 0, 0]);
observe("reserved-small",&[0]);
observe("reserved-small11",&[88]);
observe("reserved-large0",&[128, 0, 0]);
observe("reserved-large3",&[131, 0, 0]);
observe("reserved-large20",&[148, 0, 0]);
observe("irq-short",&[33, 0]);
observe("irq-long",&[36, 0, 0, 1, 0]);
observe("irq-invalid-polarity",&[35, 1, 0, 0]);
observe("irq-reserved",&[35, 1, 0, 65]);
observe("dma-reserved",&[42, 1, 3]);
observe("dma-flag-reserved",&[42, 1, 128]);
observe("dma-short",&[41, 1]);
observe("io-reserved",&[71, 128, 0, 0, 0, 0, 0, 1]);
observe("fixed-short",&[134, 8, 0, 1, 1, 1, 1, 1, 1, 1, 1]);
observe("fixed-long",&[134, 10, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
observe("address-short",&[136, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
observe("address-reserved-kind",&[136, 13, 0, 3, 12, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1]);
observe("address-reserved-flags",&[136, 13, 0, 0, 128, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1]);
observe("address-unterminated",&[136, 15, 0, 0, 12, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1, 0, 65]);
observe("address-source-index-only",&[136, 14, 0, 0, 12, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1, 0]);
observe("address-source-inner-nul",&[136, 18, 0, 0, 12, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1, 0, 65, 0, 65, 0]);
observe("address-source-nonascii",&[136, 16, 0, 0, 12, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1, 0, 128, 0]);
observe("extended-zero",&[137, 6, 0, 0, 0, 0, 0, 0, 0]);
observe("extended-truncated-table",&[137, 6, 0, 0, 2, 17, 0, 0, 0]);
observe("extended-reserved",&[137, 6, 0, 128, 1, 17, 0, 0, 0]);
observe("extended-source-bad",&[137, 8, 0, 0, 1, 17, 0, 0, 0, 0, 65]);
observe("endtag-short",&[120]);
observe("endtag-long",&[122, 0, 0]);
observe("address-pcc",&[138, 43, 0, 10, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
observe("address-vendor",&[138, 43, 0, 192, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
observe("empty",&[]);
observe("large-header-one",&[134]);
observe("large-header-two",&[134, 9]);
observe("large-payload-truncated",&[134, 9, 0, 1]);
observe("small-payload-truncated",&[34, 1]);
observe("length-max",&[134, 255, 255]);
observe("offset-nonzero",&[34, 64, 0]);
observe("endtag",&[121, 0]);
observe("endtag-nonzero",&[121, 135]);
observe("template-empty",&[]);
observe("template-only-end",&[121, 0]);
observe("template-checksum",&[121, 135]);
observe("template-bad-checksum",&[121, 1]);
observe("template-trailing",&[121, 0, 0]);
observe("template-missing",&[34, 64, 0]);
observe("template-mixture",&[113, 0, 34, 64, 0, 42, 4, 0, 121, 0]);
observe("template-later-bad",&[34, 64, 0, 0]);
observe("template-later-short",&[34, 64, 0, 134]);
observe("template-source",&[136, 16, 0, 0, 12, 0, 0, 0, 0, 1, 255, 1, 0, 0, 0, 1, 0, 65, 0, 121, 0]);
observe("template-checked-mixture",&[113, 9, 34, 64, 0, 121, 171]);
observe("last-byte-truncated",&[121]);
observe("last-two-bytes-end",&[121, 0]);
observe("extended-255",&[137, 254, 3, 7, 255, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0, 10, 0, 0, 0, 11, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 15, 0, 0, 0, 16, 0, 0, 0, 17, 0, 0, 0, 18, 0, 0, 0, 19, 0, 0, 0, 20, 0, 0, 0, 21, 0, 0, 0, 22, 0, 0, 0, 23, 0, 0, 0, 24, 0, 0, 0, 25, 0, 0, 0, 26, 0, 0, 0, 27, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 31, 0, 0, 0, 32, 0, 0, 0, 33, 0, 0, 0, 34, 0, 0, 0, 35, 0, 0, 0, 36, 0, 0, 0, 37, 0, 0, 0, 38, 0, 0, 0, 39, 0, 0, 0, 40, 0, 0, 0, 41, 0, 0, 0, 42, 0, 0, 0, 43, 0, 0, 0, 44, 0, 0, 0, 45, 0, 0, 0, 46, 0, 0, 0, 47, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 50, 0, 0, 0, 51, 0, 0, 0, 52, 0, 0, 0, 53, 0, 0, 0, 54, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 57, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 60, 0, 0, 0, 61, 0, 0, 0, 62, 0, 0, 0, 63, 0, 0, 0, 64, 0, 0, 0, 65, 0, 0, 0, 66, 0, 0, 0, 67, 0, 0, 0, 68, 0, 0, 0, 69, 0, 0, 0, 70, 0, 0, 0, 71, 0, 0, 0, 72, 0, 0, 0, 73, 0, 0, 0, 74, 0, 0, 0, 75, 0, 0, 0, 76, 0, 0, 0, 77, 0, 0, 0, 78, 0, 0, 0, 79, 0, 0, 0, 80, 0, 0, 0, 81, 0, 0, 0, 82, 0, 0, 0, 83, 0, 0, 0, 84, 0, 0, 0, 85, 0, 0, 0, 86, 0, 0, 0, 87, 0, 0, 0, 88, 0, 0, 0, 89, 0, 0, 0, 90, 0, 0, 0, 91, 0, 0, 0, 92, 0, 0, 0, 93, 0, 0, 0, 94, 0, 0, 0, 95, 0, 0, 0, 96, 0, 0, 0, 97, 0, 0, 0, 98, 0, 0, 0, 99, 0, 0, 0, 100, 0, 0, 0, 101, 0, 0, 0, 102, 0, 0, 0, 103, 0, 0, 0, 104, 0, 0, 0, 105, 0, 0, 0, 106, 0, 0, 0, 107, 0, 0, 0, 108, 0, 0, 0, 109, 0, 0, 0, 110, 0, 0, 0, 111, 0, 0, 0, 112, 0, 0, 0, 113, 0, 0, 0, 114, 0, 0, 0, 115, 0, 0, 0, 116, 0, 0, 0, 117, 0, 0, 0, 118, 0, 0, 0, 119, 0, 0, 0, 120, 0, 0, 0, 121, 0, 0, 0, 122, 0, 0, 0, 123, 0, 0, 0, 124, 0, 0, 0, 125, 0, 0, 0, 126, 0, 0, 0, 127, 0, 0, 0, 128, 0, 0, 0, 129, 0, 0, 0, 130, 0, 0, 0, 131, 0, 0, 0, 132, 0, 0, 0, 133, 0, 0, 0, 134, 0, 0, 0, 135, 0, 0, 0, 136, 0, 0, 0, 137, 0, 0, 0, 138, 0, 0, 0, 139, 0, 0, 0, 140, 0, 0, 0, 141, 0, 0, 0, 142, 0, 0, 0, 143, 0, 0, 0, 144, 0, 0, 0, 145, 0, 0, 0, 146, 0, 0, 0, 147, 0, 0, 0, 148, 0, 0, 0, 149, 0, 0, 0, 150, 0, 0, 0, 151, 0, 0, 0, 152, 0, 0, 0, 153, 0, 0, 0, 154, 0, 0, 0, 155, 0, 0, 0, 156, 0, 0, 0, 157, 0, 0, 0, 158, 0, 0, 0, 159, 0, 0, 0, 160, 0, 0, 0, 161, 0, 0, 0, 162, 0, 0, 0, 163, 0, 0, 0, 164, 0, 0, 0, 165, 0, 0, 0, 166, 0, 0, 0, 167, 0, 0, 0, 168, 0, 0, 0, 169, 0, 0, 0, 170, 0, 0, 0, 171, 0, 0, 0, 172, 0, 0, 0, 173, 0, 0, 0, 174, 0, 0, 0, 175, 0, 0, 0, 176, 0, 0, 0, 177, 0, 0, 0, 178, 0, 0, 0, 179, 0, 0, 0, 180, 0, 0, 0, 181, 0, 0, 0, 182, 0, 0, 0, 183, 0, 0, 0, 184, 0, 0, 0, 185, 0, 0, 0, 186, 0, 0, 0, 187, 0, 0, 0, 188, 0, 0, 0, 189, 0, 0, 0, 190, 0, 0, 0, 191, 0, 0, 0, 192, 0, 0, 0, 193, 0, 0, 0, 194, 0, 0, 0, 195, 0, 0, 0, 196, 0, 0, 0, 197, 0, 0, 0, 198, 0, 0, 0, 199, 0, 0, 0, 200, 0, 0, 0, 201, 0, 0, 0, 202, 0, 0, 0, 203, 0, 0, 0, 204, 0, 0, 0, 205, 0, 0, 0, 206, 0, 0, 0, 207, 0, 0, 0, 208, 0, 0, 0, 209, 0, 0, 0, 210, 0, 0, 0, 211, 0, 0, 0, 212, 0, 0, 0, 213, 0, 0, 0, 214, 0, 0, 0, 215, 0, 0, 0, 216, 0, 0, 0, 217, 0, 0, 0, 218, 0, 0, 0, 219, 0, 0, 0, 220, 0, 0, 0, 221, 0, 0, 0, 222, 0, 0, 0, 223, 0, 0, 0, 224, 0, 0, 0, 225, 0, 0, 0, 226, 0, 0, 0, 227, 0, 0, 0, 228, 0, 0, 0, 229, 0, 0, 0, 230, 0, 0, 0, 231, 0, 0, 0, 232, 0, 0, 0, 233, 0, 0, 0, 234, 0, 0, 0, 235, 0, 0, 0, 236, 0, 0, 0, 237, 0, 0, 0, 238, 0, 0, 0, 239, 0, 0, 0, 240, 0, 0, 0, 241, 0, 0, 0, 242, 0, 0, 0, 243, 0, 0, 0, 244, 0, 0, 0, 245, 0, 0, 0, 246, 0, 0, 0, 247, 0, 0, 0, 248, 0, 0, 0, 249, 0, 0, 0, 250, 0, 0, 0, 251, 0, 0, 0, 252, 0, 0, 0, 253, 0, 0, 0, 254, 0, 0, 0]);
observe("address-distinct64",&[138, 43, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 127, 0, 0, 0, 0, 0, 0, 0, 128, 255, 255, 255, 255, 255, 255, 255, 255, 16, 50, 84, 118, 152, 186, 220, 254, 240, 222, 188, 154, 120, 86, 52, 18]);
}
